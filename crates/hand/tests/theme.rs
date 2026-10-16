use cursor_core::Alias;
use hand_cursors::Hand;

#[test]
fn generated_theme_is_complete_and_consistent() {
    cursor_core::check::theme::<Hand>();
}

#[test]
fn aliased_theme_is_complete_and_consistent() {
    let aliases = Alias::parse::<Hand>("pointer=middle-finger,kill=middle-finger,text=default").unwrap();
    cursor_core::check::aliased::<Hand>(&aliases);
}
