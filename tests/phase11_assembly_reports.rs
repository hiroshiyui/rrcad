// Phase 11 Track B — Assembly intelligence.
//
// Tests for the reporting layer built on top of the assembly solver:
//   Assembly#components      — unified enumeration of placed + solved parts
//   Assembly#interferences   — clash detection with optional clearance check
//   Assembly#clash?
//   Assembly#bom / #bom_text — bill of materials with quantity rollup
//   Assembly#mass_properties — volume / mass / centre-of-mass rollup

use rrcad::ruby::vm::MrubyVm;

/// Evaluate a script and return its trimmed final value.
fn eval(src: &str) -> String {
    let mut vm = MrubyVm::new();
    vm.eval(src)
        .unwrap_or_else(|e| panic!("eval failed: {e}\n--- script ---\n{src}"))
        .trim()
        .to_string()
}

/// Evaluate a script whose final value is a String, and return the string
/// itself. The VM renders the final value with `inspect`, so a String arrives
/// quoted and escaped; undo that so tests can compare against plain text.
fn eval_str(src: &str) -> String {
    let out = eval(src);
    let inner = out
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or_else(|| panic!("expected a quoted String, got: {out}"));
    inner
        .replace("\\n", "\n")
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
}

/// Evaluate a script whose final value is a number.
fn eval_num(src: &str) -> f64 {
    let out = eval(src);
    out.parse()
        .unwrap_or_else(|e| panic!("parse {out:?} as number: {e}"))
}

/// Evaluate a script expected to raise, returning the error text.
fn eval_err(src: &str) -> String {
    let mut vm = MrubyVm::new();
    match vm.eval(src) {
        Ok(value) => panic!("expected an error, got: {value}\n--- script ---\n{src}"),
        Err(e) => e,
    }
}

/// Two stacked boxes: a 40×40×10 base with a 10×10×20 post mated on top.
const STACK: &str = r#"
    base = box(40, 40, 10)
    post = box(10, 10, 20)
    asm = assembly("rig") do |a|
      a.ground :base, base, material: "aluminium"
      a.part :post, post, material: "steel" do
        mate from: :bottom, to: face(:base, :top)
      end
    end
"#;

// ---------------------------------------------------------------------------
// components
// ---------------------------------------------------------------------------

#[test]
fn components_lists_solved_parts_with_metadata() {
    let out = eval_str(&format!(
        "{STACK}
         asm.components.map {{ |c| [c[:name], c[:component], c[:material]] }}.inspect"
    ));
    assert_eq!(
        out, r#"[[:base, :base, "aluminium"], [:post, :post, "steel"]]"#,
        "expected both solver parts with their metadata, got: {out}"
    );
}

#[test]
fn components_returns_parts_in_solved_world_positions() {
    // The post is mated onto the top of the 10 mm base, so its centroid sits
    // at z = 10 + 20/2 = 20 — proof that #components solves rather than
    // handing back the unplaced input shapes.
    let z = eval_num(&format!(
        "{STACK}
         asm.components.find {{ |c| c[:name] == :post }}[:shape].centroid[2]"
    ));
    assert!(
        (z - 20.0).abs() < 1e-6,
        "expected post centroid z=20, got {z}"
    );
}

#[test]
fn components_auto_names_unnamed_placements() {
    let out = eval_str(
        "asm = assembly(\"loose\") do |a|
           a.place box(5, 5, 5)
           a.place box(5, 5, 5).translate(10, 0, 0), name: :second
           a.place box(5, 5, 5).translate(20, 0, 0)
         end
         asm.components.map { |c| c[:name] }.inspect",
    );
    assert_eq!(
        out, "[:part_1, :second, :part_3]",
        "expected positional auto-names around the explicit one, got: {out}"
    );
}

#[test]
fn components_covers_both_placed_and_solved_parts() {
    let out = eval_str(&format!(
        "{STACK}
         asm.place box(2, 2, 2).translate(100, 0, 0), name: :spare
         asm.components.map {{ |c| c[:name] }}.inspect"
    ));
    assert_eq!(
        out, "[:spare, :base, :post]",
        "expected placements first, then solver parts, got: {out}"
    );
}

// ---------------------------------------------------------------------------
// interferences / clash?
// ---------------------------------------------------------------------------

#[test]
fn flush_mate_is_not_an_interference() {
    // The whole point: a mate puts faces in contact, and contact must not be
    // reported as a collision.
    let out = eval_str(&format!("{STACK}\nasm.interferences.inspect"));
    assert_eq!(
        out, "[]",
        "expected no interference for a flush mate: {out}"
    );
    let clash = eval_str(&format!("{STACK}\nasm.clash?.inspect"));
    assert_eq!(clash, "false", "expected clash? false, got: {clash}");
}

#[test]
fn overlapping_parts_report_volume_and_centroid() {
    // A 10-cube at the origin and another shifted +5 in x share a 5×10×10
    // slab of volume 500, centred at (7.5, 5, 5).
    let out = eval_str(
        "asm = assembly(\"clash\") do |a|
           a.place box(10, 10, 10), name: :a
           a.place box(10, 10, 10).translate(5, 0, 0), name: :b
         end
         r = asm.interferences.first
         [r[:a], r[:b], r[:type], r[:volume].round(6), r[:centroid].map { |v| v.round(6) }].inspect",
    );
    assert_eq!(
        out, "[:a, :b, :interference, 500.0, [7.5, 5.0, 5.0]]",
        "unexpected interference record: {out}"
    );
    assert_eq!(
        eval_str(
            "asm = assembly(\"clash\") do |a|
               a.place box(10, 10, 10), name: :a
               a.place box(10, 10, 10).translate(5, 0, 0), name: :b
             end
             asm.clash?.inspect"
        ),
        "true"
    );
}

#[test]
fn disjoint_parts_report_nothing_by_default() {
    let out = eval_str(
        "asm = assembly(\"apart\") do |a|
           a.place box(10, 10, 10), name: :a
           a.place box(10, 10, 10).translate(30, 0, 0), name: :b
         end
         asm.interferences.inspect",
    );
    assert_eq!(out, "[]", "expected no findings for disjoint parts: {out}");
}

#[test]
fn clearance_flags_a_gap_that_is_too_small() {
    // Boxes 20 mm apart along x with a 10 mm box each ⇒ a 10 mm gap.
    let out = eval_str(
        "asm = assembly(\"gap\") do |a|
           a.place box(10, 10, 10), name: :a
           a.place box(10, 10, 10).translate(20, 0, 0), name: :b
         end
         r = asm.interferences(clearance: 15.0).first
         [r[:a], r[:b], r[:type], r[:distance].round(6), r[:minimum]].inspect",
    );
    assert_eq!(
        out, "[:a, :b, :clearance, 10.0, 15.0]",
        "unexpected clearance record: {out}"
    );
}

#[test]
fn clearance_passes_when_the_gap_is_large_enough() {
    let out = eval_str(
        "asm = assembly(\"gap\") do |a|
           a.place box(10, 10, 10), name: :a
           a.place box(10, 10, 10).translate(20, 0, 0), name: :b
         end
         asm.interferences(clearance: 5.0).inspect",
    );
    assert_eq!(out, "[]", "10 mm gap should satisfy a 5 mm minimum: {out}");
}

#[test]
fn clearance_ignores_deliberate_contact_by_default() {
    // Every mated pair touches; flagging that under `clearance:` would bury
    // the real findings.
    let out = eval_str(&format!(
        "{STACK}\nasm.interferences(clearance: 5.0).inspect"
    ));
    assert_eq!(out, "[]", "mated contact should be ignored: {out}");
}

#[test]
fn ignore_contact_false_reports_touching_parts() {
    let out = eval_str(&format!(
        "{STACK}
         r = asm.interferences(clearance: 5.0, ignore_contact: false).first
         [r[:type], r[:distance], r[:minimum]].inspect"
    ));
    assert_eq!(
        out, "[:clearance, 0.0, 5.0]",
        "expected contact reported when asked: {out}"
    );
}

#[test]
fn interference_outranks_clearance_for_the_same_pair() {
    // An overlapping pair also has distance 0; it must be reported once, as
    // the more serious finding.
    let out = eval_str(
        "asm = assembly(\"clash\") do |a|
           a.place box(10, 10, 10), name: :a
           a.place box(10, 10, 10).translate(5, 0, 0), name: :b
         end
         asm.interferences(clearance: 20.0, ignore_contact: false)
            .map { |r| r[:type] }.inspect",
    );
    assert_eq!(out, "[:interference]", "expected a single finding: {out}");
}

#[test]
fn findings_are_sorted_worst_first() {
    // Two overlaps of different size, plus one under-clearance pair.
    let out = eval_str(
        "asm = assembly(\"mixed\") do |a|
           a.place box(10, 10, 10), name: :a
           a.place box(10, 10, 10).translate(9, 0, 0), name: :small
           a.place box(10, 10, 10).translate(2, 0, 0), name: :big
           a.place box(10, 10, 10).translate(0, 25, 0), name: :near
         end
         asm.interferences(clearance: 20.0).map { |r| [r[:a], r[:b]] }.inspect",
    );
    // :a∩:big (800) precedes :a∩:small (100) and every other overlap, and the
    // clearance findings come last.
    assert!(
        out.starts_with("[[:a, :big]"),
        "expected the largest overlap first, got: {out}"
    );
    assert!(
        out.contains(":near"),
        "expected the clearance pair reported too, got: {out}"
    );
}

#[test]
fn interferences_rejects_negative_clearance() {
    let err = eval_err("assembly(\"x\") { |a| a.place box(1, 1, 1) }.interferences(clearance: -1)");
    assert!(
        err.contains("clearance must be >= 0"),
        "unexpected error: {err}"
    );
}

// ---------------------------------------------------------------------------
// bom
// ---------------------------------------------------------------------------

#[test]
fn bom_rolls_identical_components_up_into_one_row() {
    let out = eval_str(
        "screw = cylinder(1.5, 10)
         asm = assembly(\"panel\") do |a|
           a.place box(60, 40, 5), name: :plate, material: \"aluminium\"
           4.times do |i|
             a.place screw.translate(10 + i * 12, 20, 5), name: :\"screw_#{i}\",
                     component: :m3_screw, material: \"stainless\"
           end
         end
         r = asm.bom.first
         [r[:component], r[:quantity], r[:parts].length].inspect",
    );
    assert_eq!(
        out, "[:m3_screw, 4, 4]",
        "expected four screws rolled into one row: {out}"
    );
}

#[test]
fn bom_totals_volume_and_mass_across_the_quantity() {
    // 4 × cylinder(r=1.5, h=10) = 4 × 70.6858 mm³, stainless at 8.00 g/cm³.
    let volume = eval_num(
        "screw = cylinder(1.5, 10)
         asm = assembly(\"panel\") do |a|
           4.times { |i| a.place screw.translate(i * 12, 0, 0), component: :m3, material: \"stainless\" }
         end
         asm.bom.first[:volume]",
    );
    assert!(
        (volume - 282.7433).abs() < 1e-3,
        "expected 4 × 70.686 = 282.743 mm³, got {volume}"
    );
    let mass = eval_num(
        "screw = cylinder(1.5, 10)
         asm = assembly(\"panel\") do |a|
           4.times { |i| a.place screw.translate(i * 12, 0, 0), component: :m3, material: \"stainless\" }
         end
         asm.bom.first[:mass]",
    );
    assert!(
        (mass - 2.2619).abs() < 1e-3,
        "expected 282.743 mm³ × 8.00 g/cm³ / 1000 = 2.262 g, got {mass}"
    );
}

#[test]
fn bom_sorts_by_descending_quantity_then_name() {
    let out = eval_str(
        "asm = assembly(\"panel\") do |a|
           a.place box(10, 10, 10), name: :zeta
           a.place box(10, 10, 10).translate(20, 0, 0), name: :alpha
           2.times { |i| a.place box(2, 2, 2).translate(0, 40 + i * 5, 0), component: :nut }
         end
         asm.bom.map { |r| [r[:component], r[:quantity]] }.inspect",
    );
    assert_eq!(
        out, "[[:nut, 2], [:alpha, 1], [:zeta, 1]]",
        "expected quantity-descending then alphabetical: {out}"
    );
}

#[test]
fn bom_uses_explicit_density_over_the_material_table() {
    let density = eval_num(
        "asm = assembly(\"x\") do |a|
           a.place box(10, 10, 10), name: :b, material: \"steel\", density: 1.5
         end
         asm.bom.first[:density]",
    );
    assert!(
        (density - 1.5).abs() < 1e-9,
        "expected the explicit 1.5, got {density}"
    );
}

#[test]
fn bom_falls_back_to_the_default_density_for_unknown_materials() {
    // Free-text material names are normal; an unknown one must not raise.
    let density = eval_num(
        "asm = assembly(\"x\") do |a|
           a.place box(10, 10, 10), name: :b, material: \"unobtainium\"
         end
         asm.bom(density: 2.0).first[:density]",
    );
    assert!(
        (density - 2.0).abs() < 1e-9,
        "expected the caller's fallback 2.0, got {density}"
    );
}

#[test]
fn material_names_normalize_before_the_density_lookup() {
    let density = eval_num(
        "asm = assembly(\"x\") do |a|
           a.place box(10, 10, 10), name: :b, material: \"Stainless Steel\"
         end
         asm.bom.first[:density]",
    );
    assert!(
        (density - 8.0).abs() < 1e-9,
        "expected 'Stainless Steel' to match 'stainlesssteel' (8.00), got {density}"
    );
}

#[test]
fn bom_rejects_grouping_parts_of_different_size() {
    // Silently averaging two different parts under one BOM line would produce
    // a plausible but wrong unit mass, so this is an error.
    let err = eval_err(
        "asm = assembly(\"x\") do |a|
           a.place box(10, 10, 10), component: :widget
           a.place box(20, 20, 20).translate(50, 0, 0), component: :widget
         end
         asm.bom",
    );
    assert!(
        err.contains("different volume") && err.contains("widget"),
        "unexpected error: {err}"
    );
}

#[test]
fn bom_text_renders_an_aligned_table_with_totals() {
    let out = eval_str(
        "asm = assembly(\"panel\") do |a|
           a.place box(10, 10, 10), name: :plate, material: \"aluminium\"
           2.times { |i| a.place box(2, 2, 2).translate(0, 20 + i * 5, 0), component: :nut }
         end
         asm.bom_text",
    );
    let lines: Vec<&str> = out.lines().collect();
    assert!(
        lines[0].starts_with("Item  Component"),
        "expected a header row, got: {out}"
    );
    // Every rendered row must share the rule's width, which is the whole
    // point of measuring the columns.
    let rule_len = lines[1].len();
    for line in &lines[2..] {
        assert!(
            line.len() <= rule_len,
            "row wider than the rule ({rule_len}):\n{out}"
        );
    }
    let total = lines.last().expect("a total row");
    assert!(
        total.contains("TOTAL") && total.contains('3'),
        "expected a TOTAL row summing 3 items, got: {total}"
    );
}

#[test]
fn bom_text_reports_an_empty_assembly_plainly() {
    let out = eval_str("assembly(\"empty\").bom_text");
    assert!(
        out.contains("no components"),
        "expected a plain empty message, got: {out}"
    );
}

// ---------------------------------------------------------------------------
// mass_properties
// ---------------------------------------------------------------------------

#[test]
fn mass_properties_sums_volume_over_every_component() {
    // 40×40×10 = 16000 plus 10×10×20 = 2000.
    let volume = eval_num(&format!("{STACK}\nasm.mass_properties[:volume]"));
    assert!(
        (volume - 18000.0).abs() < 1e-6,
        "expected 18000 mm³, got {volume}"
    );
}

#[test]
fn mass_properties_uses_per_part_material_densities() {
    // aluminium 16000 mm³ × 2.70 / 1000 = 43.2 g; steel 2000 × 7.85 / 1000 = 15.7 g.
    let mass = eval_num(&format!("{STACK}\nasm.mass_properties[:mass]"));
    assert!(
        (mass - 58.9).abs() < 1e-6,
        "expected 43.2 + 15.7 = 58.9 g, got {mass}"
    );
}

#[test]
fn centre_of_mass_is_weighted_by_mass_not_volume() {
    // Base centroid z = 5 (43.2 g), post centroid z = 20 (15.7 g):
    //   (5×43.2 + 20×15.7) / 58.9 = 8.99830…
    // A volume-weighted mean would give 6.667, and an unweighted one 12.5.
    let z = eval_num(&format!("{STACK}\nasm.mass_properties[:center_of_mass][2]"));
    assert!(
        (z - 8.998302).abs() < 1e-5,
        "expected mass-weighted z ≈ 8.9983, got {z}"
    );
}

#[test]
fn centre_of_mass_of_a_symmetric_pair_sits_between_them() {
    let out = eval_str(
        "asm = assembly(\"pair\") do |a|
           a.place box(10, 10, 10), name: :left
           a.place box(10, 10, 10).translate(30, 0, 0), name: :right
         end
         asm.mass_properties[:center_of_mass].map { |v| v.round(6) }.inspect",
    );
    assert_eq!(
        out, "[20.0, 5.0, 5.0]",
        "expected the midpoint of centroids 5 and 35, got: {out}"
    );
}

#[test]
fn mass_properties_breaks_the_rollup_down_per_part() {
    let out = eval_str(&format!(
        "{STACK}
         asm.mass_properties[:parts].map {{ |p| [p[:name], p[:density], p[:mass].round(6)] }}.inspect"
    ));
    assert_eq!(
        out, "[[:base, 2.7, 43.2], [:post, 7.85, 15.7]]",
        "expected a per-part breakdown echoing the density used: {out}"
    );
}

#[test]
fn mass_properties_default_density_matches_mass_estimate() {
    // An assembly of one unadorned part must agree with Kernel#mass_estimate,
    // which is the single-part version of the same calculation.
    let delta = eval_num(
        "part = box(10, 10, 10)
         asm = assembly(\"one\") { |a| a.place part }
         (asm.mass_properties[:mass] - mass_estimate(part)).abs",
    );
    assert!(delta < 1e-9, "expected agreement, differed by {delta}");
}

#[test]
fn mass_properties_rejects_an_empty_assembly() {
    let err = eval_err("assembly(\"empty\").mass_properties");
    assert!(
        err.contains("no shapes"),
        "unexpected error for an empty assembly: {err}"
    );
}

#[test]
fn mass_properties_rejects_a_non_positive_density() {
    let err = eval_err("assembly(\"x\") { |a| a.place box(1, 1, 1) }.mass_properties(density: 0)");
    assert!(err.contains("must be > 0"), "unexpected error: {err}");
}

#[test]
fn part_metadata_rejects_a_non_positive_density() {
    let err = eval_err("assembly(\"x\") { |a| a.place box(1, 1, 1), density: -1 }");
    assert!(err.contains("must be > 0"), "unexpected error: {err}");
}

#[test]
fn part_metadata_rejects_a_non_string_material() {
    let err = eval_err("assembly(\"x\") { |a| a.place box(1, 1, 1), material: 42 }");
    assert!(
        err.contains("material must be a String or Symbol"),
        "unexpected error: {err}"
    );
}

// ---------------------------------------------------------------------------
// Interaction with the rest of the assembly API
// ---------------------------------------------------------------------------

#[test]
fn reporting_does_not_disturb_to_shape() {
    // The reports solve the assembly; that must not consume or mutate it.
    let volume = eval_num(&format!(
        "{STACK}
         asm.interferences
         asm.bom
         asm.mass_properties
         asm.to_shape.volume"
    ));
    assert!(
        (volume - 18000.0).abs() < 1e-6,
        "expected to_shape still to yield 18000 mm³, got {volume}"
    );
}

#[test]
fn metadata_keywords_work_on_the_placement_helpers() {
    let out = eval_str(
        "base = box(20, 20, 5)
         post = box(4, 4, 8)
         asm = assembly(\"rig\") do |a|
           a.place base, name: :base
           a.mate post, from: post.faces(:bottom).first, to: base.faces(:top).first,
                  name: :post, material: \"brass\"
         end
         asm.bom.map { |r| [r[:component], r[:material]] }.inspect",
    );
    assert_eq!(
        out, r#"[[:base, nil], [:post, "brass"]]"#,
        "expected #mate to carry reporting metadata too: {out}"
    );
}

#[test]
fn an_under_constrained_assembly_still_reports_its_solver_error() {
    let err = eval_err(
        "asm = assembly(\"rig\") do |a|
           a.ground :base, box(10, 10, 10)
           a.part :floater, box(2, 2, 2)
         end
         asm.bom",
    );
    assert!(
        err.contains("under-constrained"),
        "expected the solver error to surface through #bom, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// mass: override — datasheet weight for parts you buy rather than model
// ---------------------------------------------------------------------------

#[test]
fn stated_mass_overrides_volume_times_density() {
    // A 10 mm cube of "aluminium" would compute 2.7 g; the datasheet says 182.
    let mass = eval_num(
        "asm = assembly(\"x\") do |a|
           a.place box(10, 10, 10), name: :battery, mass: 182.0
         end
         asm.mass_properties[:mass]",
    );
    assert!(
        (mass - 182.0).abs() < 1e-9,
        "expected the stated 182 g, got {mass}"
    );
}

#[test]
fn mass_source_distinguishes_stated_from_computed() {
    let out = eval_str(
        "asm = assembly(\"x\") do |a|
           a.place box(10, 10, 10), name: :modelled, material: \"aluminium\"
           a.place box(10, 10, 10).translate(50, 0, 0), name: :bought, mass: 182.0
         end
         asm.mass_properties[:parts].map { |p| p[:mass_source] }.inspect",
    );
    assert_eq!(
        out, "[:density, :stated]",
        "expected each row to say where its mass came from: {out}"
    );
}

#[test]
fn stated_mass_back_computes_an_effective_density() {
    // 182 g in a 1 cm³ envelope is 182 g/cm³ — implausible, and that is the
    // point: the number surfaces an envelope that is the wrong size.
    let density = eval_num(
        "asm = assembly(\"x\") do |a|
           a.place box(10, 10, 10), name: :battery, mass: 182.0
         end
         asm.mass_properties[:parts][0][:density]",
    );
    assert!(
        (density - 182.0).abs() < 1e-9,
        "expected an effective density of 182 g/cm³, got {density}"
    );
}

#[test]
fn stated_mass_shifts_the_centre_of_mass() {
    // Two identical cubes, 1 g and 99 g: the CoM must sit at 104, not 55.
    let x = eval_num(
        "asm = assembly(\"x\") do |a|
           a.place box(10, 10, 10), name: :light, mass: 1.0
           a.place box(10, 10, 10).translate(100, 0, 0), name: :heavy, mass: 99.0
         end
         asm.mass_properties[:center_of_mass][0]",
    );
    assert!(
        (x - 104.0).abs() < 1e-9,
        "expected a mass-weighted 104.0, got {x}"
    );
}

#[test]
fn mass_and_density_together_are_rejected() {
    // Two answers to the same question; preferring one silently would hide it.
    let err = eval_err("assembly(\"x\") { |a| a.place box(1, 1, 1), mass: 5.0, density: 2.0 }");
    assert!(
        err.contains("either mass: or density:"),
        "unexpected error: {err}"
    );
}

#[test]
fn mass_rejects_a_non_positive_value() {
    let err = eval_err("assembly(\"x\") { |a| a.place box(1, 1, 1), mass: 0 }");
    assert!(err.contains("must be > 0"), "unexpected error: {err}");
}

#[test]
fn bom_rolls_up_stated_masses() {
    // Four motors at 32.5 g each.
    let mass = eval_num(
        "motor = cylinder(14, 12)
         asm = assembly(\"quad\") do |a|
           4.times do |i|
             a.place motor.translate(i * 60, 0, 0), name: :\"motor_#{i}\",
                     component: :motor_2207, mass: 32.5
           end
         end
         asm.bom.first[:mass]",
    );
    assert!(
        (mass - 130.0).abs() < 1e-9,
        "expected 4 × 32.5 = 130 g, got {mass}"
    );
}

#[test]
fn bom_rejects_grouping_parts_of_different_stated_mass() {
    let err = eval_err(
        "asm = assembly(\"x\") do |a|
           a.place box(10, 10, 10), component: :cell, mass: 40.0
           a.place box(10, 10, 10).translate(20, 0, 0), component: :cell, mass: 55.0
         end
         asm.bom",
    );
    assert!(
        err.contains("different mass") && err.contains("cell"),
        "unexpected error: {err}"
    );
}

#[test]
fn an_overridden_part_still_takes_part_in_clash_checks() {
    // The envelope is real geometry — that is why we keep it.
    let out = eval_str(
        "asm = assembly(\"x\") do |a|
           a.place box(10, 10, 10), name: :frame
           a.place box(10, 10, 10).translate(5, 0, 0), name: :battery, mass: 182.0
         end
         asm.interferences.map { |r| [r[:a], r[:b], r[:volume]] }.inspect",
    );
    assert_eq!(
        out, "[[:frame, :battery, 500.0]]",
        "expected the overridden part to still clash-check: {out}"
    );
}

#[test]
fn a_zero_volume_shape_with_a_stated_mass_acts_as_a_point_mass() {
    // Falls out of the design rather than being special-cased: a datum plane
    // has no volume, so it contributes only its stated mass at its position.
    // 1 g body at z=5 plus 3 g at z=50 ⇒ CoM z = 38.75.
    let z = eval_num(
        "plane = datum_plane(origin: [0, 0, 50], normal: [0, 0, 1], x_dir: [1, 0, 0])
         asm = assembly(\"pm\") do |a|
           a.place box(10, 10, 10), name: :body, density: 1.0
           a.place plane, name: :wiring, mass: 3.0
         end
         asm.mass_properties[:center_of_mass][2]",
    );
    assert!(
        (z - 38.75).abs() < 1e-9,
        "expected a point mass to pull the CoM to 38.75, got {z}"
    );
    // A shape with no volume has no meaningful density.
    let out = eval_str(
        "plane = datum_plane(origin: [0, 0, 50], normal: [0, 0, 1], x_dir: [1, 0, 0])
         asm = assembly(\"pm\") do |a|
           a.place box(10, 10, 10), name: :body, density: 1.0
           a.place plane, name: :wiring, mass: 3.0
         end
         asm.mass_properties[:parts][1][:density].inspect",
    );
    assert_eq!(
        out, "nil",
        "expected nil density for a zero-volume part: {out}"
    );
}

// ---------------------------------------------------------------------------
// Inertia rollup
// ---------------------------------------------------------------------------

/// Three disjoint boxes of one material — the fused-solid oracle fixture.
const TRIPLE: &str = r#"
    a = box(10, 10, 10)
    b = box(10, 10, 10).translate(10, 10, 0)
    c = box(20, 5, 5).translate(0, 0, 30)
    asm = assembly("t") do |x|
      x.place a, name: :a, material: "aluminium"
      x.place b, name: :b, material: "aluminium"
      x.place c, name: :c, material: "aluminium"
    end
    fused = a.fuse(b).fuse(c)
"#;

#[test]
fn inertia_rollup_matches_the_fused_solid() {
    // The load-bearing test. For uniform density the rollup must equal the
    // tensor OCCT computes for one fused solid — the same quantity by a
    // completely different route, so agreement is real evidence rather than
    // self-consistency. Checks all six components, signs included.
    let out = eval_str(&format!(
        "{TRIPLE}
         mp = asm.mass_properties
         scale = 2.70 / 1000.0
         oracle = fused.inertia
         [:ixx, :iyy, :izz, :ixy, :ixz, :iyz].map {{ |k|
           want = oracle[k] * scale
           got = mp[:inertia][k]
           rel = (got - want).abs / [want.abs, 1.0].max
           rel < 1.0e-9
         }}.inspect"
    ));
    assert_eq!(
        out, "[true, true, true, true, true, true]",
        "rollup disagreed with the fused solid: {out}"
    );
}

#[test]
fn rollup_centre_of_mass_matches_the_fused_solid() {
    let out = eval_str(&format!(
        "{TRIPLE}
         rollup = asm.mass_properties[:center_of_mass]
         solid = fused.centroid
         [0, 1, 2].map {{ |i| (rollup[i] - solid[i]).abs < 1.0e-9 }}.inspect"
    ));
    assert_eq!(
        out, "[true, true, true]",
        "CoM disagreed with the fused solid: {out}"
    );
}

#[test]
fn inertia_about_the_centre_of_mass_matches_the_analytic_box() {
    // Solid box about its own centre: Ixx = m(b² + c²)/12.
    let ixx = eval_num(
        "asm = assembly(\"b\") { |x| x.place box(10, 10, 10), name: :b, density: 1.0 }
         asm.mass_properties[:inertia][:ixx]",
    );
    assert!(
        (ixx - 1.0 * 200.0 / 12.0).abs() < 1e-9,
        "expected m(b²+c²)/12 = 16.667, got {ixx}"
    );
}

#[test]
fn inertia_about_the_origin_matches_the_analytic_corner_box() {
    // Same box referenced to a corner: Ixx = m(b² + c²)/3 — four times the
    // centre value, which is what the parallel-axis term must supply.
    let ixx = eval_num(
        "asm = assembly(\"b\") { |x| x.place box(10, 10, 10), name: :b, density: 1.0 }
         asm.mass_properties(about: :origin)[:inertia][:ixx]",
    );
    assert!(
        (ixx - 1.0 * 200.0 / 3.0).abs() < 1e-9,
        "expected m(b²+c²)/3 = 66.667, got {ixx}"
    );
}

#[test]
fn inertia_accepts_an_explicit_reference_point() {
    // About (0,0,100): the transfer term dominates — m(dy² + dz²) with the
    // box centre at (5,5,5) ⇒ 1 × (25 + 9025) = 9050, plus 16.667 own.
    let ixx = eval_num(
        "asm = assembly(\"b\") { |x| x.place box(10, 10, 10), name: :b, density: 1.0 }
         asm.mass_properties(about: [0, 0, 100])[:inertia][:ixx]",
    );
    assert!(
        (ixx - (9050.0 + 200.0 / 12.0)).abs() < 1e-6,
        "expected 9066.667, got {ixx}"
    );
}

#[test]
fn inertia_about_is_reported_back() {
    let out = eval_str(
        "asm = assembly(\"b\") { |x| x.place box(10, 10, 10), name: :b, density: 1.0 }
         asm.mass_properties(about: :origin)[:inertia_about].inspect",
    );
    assert_eq!(
        out, "[0.0, 0.0, 0.0]",
        "expected the reference echoed: {out}"
    );
}

#[test]
fn cylinder_inertia_about_its_axis_matches_the_analytic_value() {
    // Solid cylinder: Izz = m·r²/2.
    let out = eval_str(
        "asm = assembly(\"c\") { |x| x.place cylinder(5, 20), name: :c, density: 1.0 }
         mp = asm.mass_properties
         ((mp[:inertia][:izz] - mp[:mass] * 25.0 / 2.0).abs < 1.0e-9).inspect",
    );
    assert_eq!(out, "true", "cylinder Izz disagreed with m·r²/2");
}

#[test]
fn inertia_about_the_centre_of_mass_is_translation_invariant() {
    // Moving the whole assembly must not change its tensor about its own CoM.
    let out = eval_str(
        "def rig(dx)
           assembly(\"r\") do |x|
             x.place box(10, 10, 10).translate(dx, 0, 0), name: :a, density: 1.0
             x.place box(4, 4, 4).translate(dx + 30, 0, 0), name: :b, density: 1.0
           end
         end
         near = rig(0).mass_properties[:inertia]
         far = rig(500).mass_properties[:inertia]
         [:ixx, :iyy, :izz].map { |k| (near[k] - far[k]).abs < 1.0e-6 }.inspect",
    );
    assert_eq!(
        out, "[true, true, true]",
        "tensor moved with the assembly: {out}"
    );
}

#[test]
fn off_diagonal_entries_are_tensor_entries_not_products_of_inertia() {
    // Two unit-density 10-cubes on a diagonal about their shared CoM:
    // ∫xy dV = +50000, so a true tensor entry (−∫xy dV) must be −50.0 g·mm²
    // at density 1. The sign is the whole point — getting it backwards would
    // silently mirror the coupling terms.
    let ixy = eval_num(
        "asm = assembly(\"d\") do |x|
           x.place box(10, 10, 10), name: :a, density: 1.0
           x.place box(10, 10, 10).translate(10, 10, 0), name: :b, density: 1.0
         end
         asm.mass_properties[:inertia][:ixy]",
    );
    assert!(
        (ixy - (-50.0)).abs() < 1e-9,
        "expected ixy = -50.0 (tensor convention), got {ixy}"
    );
}

#[test]
fn inertia_scales_with_a_stated_mass() {
    // A stated mass keeps the envelope's inertia distribution and rescales it:
    // 182 g in a 10-cube ⇒ Ixx = 182 × 200/12.
    let ixx = eval_num(
        "asm = assembly(\"x\") { |a| a.place box(10, 10, 10), name: :bat, mass: 182.0 }
         asm.mass_properties[:inertia][:ixx]",
    );
    assert!(
        (ixx - 182.0 * 200.0 / 12.0).abs() < 1e-6,
        "expected 3033.333, got {ixx}"
    );
}

#[test]
fn a_symmetric_assembly_has_no_off_diagonal_coupling() {
    // Four arms in a cross, as on a quad: the products must vanish.
    let out = eval_str(
        "arm = box(40, 6, 3).translate(10, -3, 0)
         asm = assembly(\"quad\") do |a|
           4.times { |i| a.place arm.rotate(0, 0, 1, i * 90), name: :\"arm_#{i}\", density: 1.0 }
         end
         i = asm.mass_properties[:inertia]
         [:ixy, :ixz, :iyz].map { |k| i[k].abs < 1.0e-6 }.inspect",
    );
    assert_eq!(
        out, "[true, true, true]",
        "a symmetric cross should have no coupling terms: {out}"
    );
}

#[test]
fn mass_properties_does_not_leak_the_shape_handle() {
    // The rollup needs each Shape internally; the report must stay plain data.
    let out = eval_str(&format!(
        "{STACK}\nasm.mass_properties[:parts][0].key?(:shape).inspect"
    ));
    assert_eq!(out, "false", "expected no :shape key in the report: {out}");
}

#[test]
fn mass_properties_rejects_a_malformed_about_point() {
    let err =
        eval_err("assembly(\"x\") { |a| a.place box(1, 1, 1) }.mass_properties(about: [0, 0])");
    assert!(
        err.contains("about:") && err.contains("3-element"),
        "unexpected error: {err}"
    );
}

// ---------------------------------------------------------------------------
// Assembly#export options
//
// `Assembly#export` used to take only a path, silently dropping every drawing
// option, so an assembly could not produce the one deliverable it most needs —
// a sheet. Options are now forwarded to the fused Shape untouched.
// ---------------------------------------------------------------------------

/// A throwaway working directory, removed on drop. `safe_path` confines
/// exports to the process CWD, so these write into it and clean up after.
struct Workspace {
    dir: std::path::PathBuf,
}

impl Workspace {
    fn new(tag: &str) -> Self {
        let dir = std::env::current_dir()
            .expect("cwd")
            .join(format!("target/asmexport_{tag}"));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).expect("create workspace");
        Self { dir }
    }

    /// Export `asm` with `opts` and return the file's contents.
    fn export(&self, name: &str, opts: &str) -> String {
        let path = self.dir.join(name);
        let literal = format!("{:?}", path.to_string_lossy());
        let args = if opts.is_empty() {
            literal.clone()
        } else {
            format!("{literal}, {opts}")
        };
        eval(&format!(
            "asm = assembly(\"rig\") do |a|
               a.place box(40, 30, 10), name: :base
               a.place cylinder(6, 25).translate(20, 15, 10), name: :post
             end
             asm.export({args})"
        ));
        std::fs::read_to_string(&path).expect("read exported file")
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.dir).ok();
    }
}

#[test]
fn assembly_export_still_works_with_no_options() {
    let ws = Workspace::new("plain");
    let svg = ws.export("plain.svg", "");
    assert!(svg.contains("<svg"), "expected an SVG document");
}

#[test]
fn assembly_export_forwards_the_view_option() {
    let ws = Workspace::new("sheet");
    let svg = ws.export("sheet.svg", "view: :sheet, title_block: true");
    for view in ["view-top", "view-front", "view-side"] {
        assert!(svg.contains(view), "missing {view} on the sheet");
    }
}

#[test]
fn assembly_export_forwards_the_section_option() {
    let ws = Workspace::new("section");
    let svg = ws.export("section.svg", "view: :front, section: :xz");
    assert!(svg.contains("hatch\""), "expected a hatched cut face");
}

#[test]
fn assembly_export_forwards_the_annotation_options() {
    let ws = Workspace::new("annotations");
    let svg = ws.export("ann.svg", "view: :top, dimensions: true, ordinate: true");
    assert!(svg.contains("class=\"dimensions\""), "missing dimensions");
    assert!(
        svg.contains("class=\"ordinates\""),
        "missing ordinate dimensions — the post's cylinder is a located feature"
    );
}

#[test]
fn assembly_export_still_reaches_the_solid_formats() {
    let ws = Workspace::new("step");
    let step = ws.export("rig.step", "");
    assert!(step.contains("ISO-10303-21"), "expected a STEP file");
}
