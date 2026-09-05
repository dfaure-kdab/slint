// Copyright © Klarälvdalens Datakonsult AB, a KDAB Group company, info@kdab.com, author David Faure <david.faure@kdab.com>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0

//! A window the source leaves untitled gets its `title` bound to `default-title`, which the
//! runtime fills with the application name. A title the source decides is left alone, an empty
//! one included.

use i_slint_compiler::diagnostics::BuildDiagnostics;
use i_slint_compiler::expression_tree::Expression;
use i_slint_compiler::generator::OutputFormat;
use i_slint_compiler::parser::parse;
use i_slint_compiler::{CompilerConfiguration, compile_syntax_node};

/// Whether the root element's `title` is the binding to `default-title` that the compiler adds.
fn title_falls_back_to_default(source: &str) -> bool {
    let mut diagnostics = BuildDiagnostics::default();
    let syntax_node = parse(source.into(), None, &mut diagnostics);
    let compiler_config = CompilerConfiguration::new(OutputFormat::Interpreter);
    let (doc, diagnostics, _) =
        spin_on::spin_on(compile_syntax_node(syntax_node, diagnostics, compiler_config));
    assert!(!diagnostics.has_errors(), "{:?}", diagnostics.to_string_vec());
    let root = doc.last_exported_component().unwrap().root_element.clone();
    let root = root.borrow();
    root.binding_cell_including_synthetic("title").is_some_and(|binding| {
        matches!(binding.borrow().value_expression(),
            Expression::PropertyReference(nr) if nr.name() == "default-title")
    })
}

#[test]
fn an_untitled_window_falls_back() {
    assert!(title_falls_back_to_default(r#"export component T inherits Window { }"#));
}

#[test]
fn a_root_that_is_not_a_window_falls_back() {
    // ensure_window wraps it in a Window, which is untitled too
    assert!(title_falls_back_to_default(r#"export component T inherits Rectangle { }"#));
}

#[test]
fn a_title_from_the_source_wins() {
    assert!(!title_falls_back_to_default(r#"export component T inherits Window { title: "hi"; }"#));
    // An empty title is a choice too, not an absence
    assert!(!title_falls_back_to_default(r#"export component T inherits Window { title: ""; }"#));
    assert!(!title_falls_back_to_default(
        r#"export component T inherits Window {
               in-out property <string> n: "x";
               title: "Editing " + root.n;
           }"#
    ));
}

#[test]
fn a_title_from_a_base_component_wins() {
    assert!(!title_falls_back_to_default(
        r#"component Base inherits Window { title: "from the base"; }
           export component T inherits Base { }"#
    ));
}

#[test]
fn a_two_way_binding_wins() {
    // The alias survives the fallback, so the application still drives the title through `n`
    assert!(!title_falls_back_to_default(
        r#"export component T inherits Window {
               in-out property <string> n: "x";
               title <=> root.n;
           }"#
    ));
}

#[test]
fn a_system_tray_icon_root_is_left_alone() {
    // It has a `title` of its own but no window, so there is no `default-title` to bind to
    assert!(!title_falls_back_to_default(
        r#"export component T inherits SystemTrayIcon {
               icon: @image-url("");
           }"#
    ));
}

#[test]
fn a_component_may_declare_a_default_title_of_its_own() {
    // `default-title` is @shadowable, and the fallback still reads the builtin one
    assert!(title_falls_back_to_default(
        r#"export component T inherits Window {
               property <string> default-title: "mine";
               Text { text: root.default-title; }
           }"#
    ));
}

#[test]
fn a_dialog_root_falls_back() {
    assert!(title_falls_back_to_default(
        r#"export component T inherits Dialog { Text { text: "hi"; } }"#
    ));
}

/// The whole `title` binding, as a debug string.
fn title_binding(source: &str) -> String {
    let mut diagnostics = BuildDiagnostics::default();
    let syntax_node = parse(source.into(), None, &mut diagnostics);
    let compiler_config = CompilerConfiguration::new(OutputFormat::Interpreter);
    let (doc, diagnostics, _) =
        spin_on::spin_on(compile_syntax_node(syntax_node, diagnostics, compiler_config));
    assert!(!diagnostics.has_errors(), "{:?}", diagnostics.to_string_vec());
    let root = doc.last_exported_component().unwrap().root_element.clone();
    let root = root.borrow();
    format!("{:?}", root.binding_cell_including_synthetic("title").unwrap().borrow().expression)
}

#[test]
fn a_state_that_sets_the_title_keeps_the_fallback_for_its_else_branch() {
    // lower_states turns the state into `active ? "state" : <whatever title was bound to>`, so
    // the fallback has to be in place before that pass runs
    let binding = title_binding(
        r#"export component T inherits Window {
               in-out property <bool> b;
               states [ s when root.b: { title: "state"; } ]
           }"#,
    );
    assert!(binding.contains("state"), "{binding}");
    assert!(binding.contains("default-title"), "{binding}");
}

#[test]
fn a_state_in_a_base_component_keeps_the_fallback_too() {
    // The fallback lands on the base, whose root is the one inheriting the builtin
    let binding = title_binding(
        r#"component Base inherits Window {
               in-out property <bool> b;
               states [ s when root.b: { title: "state"; } ]
               TouchArea { clicked => { root.b = !root.b; } }
           }
           export component T inherits Base { }"#,
    );
    assert!(binding.contains("state"), "{binding}");
    assert!(binding.contains("default-title"), "{binding}");
}
