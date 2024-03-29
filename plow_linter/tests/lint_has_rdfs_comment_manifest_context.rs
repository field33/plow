use harriet::TurtleDocument;

use plow_linter::lints::{HasRdfsCommentManifestContext, PlowLint};
use plow_linter::Linter;

const RDFS_COMMENT_MANIFEST_CONTEXT_BASE: &str = concat!(
    include_str!("data/default_ttl_header"),
    r#"
registry:ontologyFormatVersion "v1" ;
registry:canonicalPrefix "test" ;
registry:packageVersion "=2.3.4" ;
registry:packageName "@field33/valid" ;
"#
);

#[test]
fn lint_registry_rdfs_comment_manifest_context_exists_and_valid() {
    // Only alphanumeric characters and underscored are allowed.
    let ttl_document_with_rdfs_comment_a =
        format!("{RDFS_COMMENT_MANIFEST_CONTEXT_BASE} rdfs:comment \"
Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed magna dolor, facilisis imperdiet mi vel, consectetur hendrerit tortor. Proin volutpat velit id euismod egestas. Aenean finibus vehicula enim, non feugiat eros condimentum ut. Nullam maximus semper eros eu gravida. Cras tristique luctus est, sed ullamcorper nibh pharetra a. Nullam ac ex eu sapien tempus vestibulum a sit amet lacus. Sed viverra tellus at elit lacinia egestas eu facilisis sem. Donec imperdiet eros pretium odio maximus mollis. Donec id est facilisis, blandit risus et, maximus eros. Suspendisse sit amet tincidunt eros. Donec aliquet ligula odio, non sollicitudin quam convallis eu. Sed luctus tincidunt dolor quis hendrerit. Mauris ornare sit amet arcu at molestie. Nullam tincidunt sapien tellus, efficitur commodo dui dictum nec. Nunc gravida nisi a ex faucibus, vitae hendrerit neque vulputate. Morbi purus lectus, laoreet vel ligula vitae, molestie efficitur nisi.

Donec sodales condimentum enim, non pharetra est. Aliquam viverra ornare est, ac tincidunt neque convallis et. Sed iaculis ipsum ac massa sagittis, eget auctor est pharetra. Proin tempus condimentum imperdiet. Etiam metus nibh, egestas at risus id, dignissim condimentum massa. Ut tempor odio nunc, vitae semper eros maximus vel. Maecenas commodo, magna nec elementum euismod, mi mi interdum ante, nec tempus felis sapien a eros. Sed blandit tincidunt scelerisque.

Mauris in dapibus felis. Donec quam enim, varius vel efficitur eget, interdum ac ante. Praesent fringilla, odio convallis molestie porttitor, lectus tellus tincidunt nulla, accumsan tortor. \"@la .");
    let ttl_document_with_rdfs_comment_b =
        format!("{RDFS_COMMENT_MANIFEST_CONTEXT_BASE} rdfs:comment \"
Lorem ipsum dolor sit amet, consectetur adipiscing elit. Curabitur dignissim, lacus at feugiat viverra, tortor metus tristique justo, et eleifend lectus eros a arcu. Vivamus in sagittis leo, sit amet semper velit. Phasellus tempor lacus ex, vehicula suscipit nisi sodales eget. Nulla a faucibus arcu. Aliquam a volutpat quam, vitae gravida metus. Duis nec fermentum libero. Aenean imperdiet neque eget accumsan maximus. Suspendisse consectetur ornare ipsum ac condimentum.

Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Integer dapibus enim cursus elit fringilla, eget efficitur augue gravida. Sed dui nisl, lobortis in porttitor pulvinar, aliquet at sapien. In sagittis lobortis urna, in convallis ipsum commodo ac. Nullam convallis sagittis turpis, at dapibus purus. Nullam sed congue massa, sit amet commodo massa. Nunc laoreet a ante vitae eleifend. Aliquam tristique lectus nec elit egestas ornare.

Sed ac lectus vestibulum, rutrum nisi eu, congue ex. Nam sed elit nec dolor interdum dignissim sed scelerisque neque. Pellentesque convallis nisi in felis tempor porta. Phasellus elementum sapien id posuere euismod. Suspendisse at varius nibh, eget pellentesque nulla. Morbi ultricies lacinia pharetra. In id condimentum purus. Aenean ligula tellus, luctus ut leo et, rhoncus porttitor nisl. Phasellus bibendum egestas sapien et volutpat. Pellentesque scelerisque finibus nisl sed eleifend. Duis at sem eget eros euismod fringilla placerat nec enim. Pellentesque habitant morbi tristique senectus et netus et malesuada fames ac turpis egestas. Donec accumsan, nisl quis maximus cursus, est dui ultrices risus, a sollicitudin leo turpis quis mi. Cras arcu diam, lobortis et tristique nec, pharetra et nulla.

Duis et elit ac tellus facilisis venenatis eu ut mi. Etiam dapibus rhoncus lacus nec luctus. Nunc feugiat condimentum leo, ut convallis felis tincidunt. \"@la .");
    let ttl_document_with_rdfs_comment_c =
        format!("{RDFS_COMMENT_MANIFEST_CONTEXT_BASE} rdfs:comment \"Long description with inappropriate words asshole fucking shit.\"@en .");
    let ttl_document_with_rdfs_comment_d =
        format!("{RDFS_COMMENT_MANIFEST_CONTEXT_BASE} rdfs:comment \"Multiple long descriptions.\"@en, \"Which are not allowed.\"@en .");
    let ttl_document_with_rdfs_comment_e = format!(
        "{RDFS_COMMENT_MANIFEST_CONTEXT_BASE} rdfs:comment \"A language tag is necessary.\" ."
    );
    let ttl_document_with_rdfs_comment_f =
        format!("{RDFS_COMMENT_MANIFEST_CONTEXT_BASE} rdfs:comment \"Multiple long descriptions.\", \"Which are allowed because it uses a generic annotation on the other hand only the first one will be evaulated as a long description and later ones are ignored in this case this test case should FAIL because of a missing language tag.\"@en .");

    let mut linter_a = Linter::try_from(ttl_document_with_rdfs_comment_a.as_ref()).unwrap();
    linter_a.add_lint_as_set(
        vec![Box::new(HasRdfsCommentManifestContext::default()) as PlowLint],
        None,
    );
    let mut linter_b = Linter::try_from(ttl_document_with_rdfs_comment_b.as_ref()).unwrap();
    linter_b.add_lint_as_set(
        vec![Box::new(HasRdfsCommentManifestContext::default()) as PlowLint],
        None,
    );
    let mut linter_c = Linter::try_from(ttl_document_with_rdfs_comment_c.as_ref()).unwrap();
    linter_c.add_lint_as_set(
        vec![Box::new(HasRdfsCommentManifestContext::default()) as PlowLint],
        None,
    );
    let mut linter_d = Linter::try_from(ttl_document_with_rdfs_comment_d.as_ref()).unwrap();
    linter_d.add_lint_as_set(
        vec![Box::new(HasRdfsCommentManifestContext::default()) as PlowLint],
        None,
    );
    let mut linter_e = Linter::try_from(ttl_document_with_rdfs_comment_e.as_ref()).unwrap();
    linter_e.add_lint_as_set(
        vec![Box::new(HasRdfsCommentManifestContext::default()) as PlowLint],
        None,
    );
    let mut linter_f = Linter::try_from(ttl_document_with_rdfs_comment_f.as_ref()).unwrap();
    linter_f.add_lint_as_set(
        vec![Box::new(HasRdfsCommentManifestContext::default()) as PlowLint],
        None,
    );

    let result_a = linter_a.run_all_lints();
    let result_b = linter_b.run_all_lints();
    let result_c = linter_c.run_all_lints();
    let result_d = linter_d.run_all_lints();
    let result_e = linter_e.run_all_lints();
    let result_f = linter_f.run_all_lints();
    assert!(result_a.first().unwrap().is_success());
    assert!(result_b.first().unwrap().is_failure());
    // Profanity filter turned off.
    assert!(result_c.first().unwrap().is_success());
    //
    assert!(result_d.first().unwrap().is_failure());
    assert!(result_e.first().unwrap().is_failure());
    assert!(result_f.first().unwrap().is_failure());
}

#[test]
fn lint_registry_rdfs_comment_manifest_context_does_not_exist_or_empty() {
    assert!(TurtleDocument::parse_full(RDFS_COMMENT_MANIFEST_CONTEXT_BASE).is_err());
    assert!(TurtleDocument::parse_full(&format!(
        "{RDFS_COMMENT_MANIFEST_CONTEXT_BASE} rdfs:comment \"\" ."
    ))
    .is_ok());
}
