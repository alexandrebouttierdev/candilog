use super::super::extension_pour;

#[test]
fn la_famille_linux_choisit_deb_ou_rpm() {
    assert_eq!(extension_pour("windows", &[]), Some("exe"));
    assert_eq!(extension_pour("macos", &[]), Some("dmg"));
    assert_eq!(extension_pour("linux", &["debian"]), Some("deb"));
    assert_eq!(extension_pour("linux", &["ubuntu"]), Some("deb"));
    assert_eq!(extension_pour("linux", &["fedora"]), Some("rpm"));
    assert_eq!(extension_pour("linux", &["rhel"]), Some("rpm"));
    assert_eq!(extension_pour("linux", &["centos"]), Some("rpm"));
    assert_eq!(extension_pour("linux", &["debian", "fedora"]), Some("deb"));
    assert_eq!(extension_pour("linux", &[]), None);
    assert_eq!(extension_pour("linux", &["arch", "nixos"]), None);
    assert_eq!(extension_pour("plan9", &[]), None);
}
