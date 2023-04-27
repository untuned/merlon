use std::io::prelude::*;
use std::fs::File;

#[test]
fn sync_complex_dependency_graph_to_repo() -> Result<()> {
    let tempdir = TempDir::new()?;
    let dir_path = tempdir.path();
    let mut registry = Registry::new();

    // Helper function to create a package with one patch and register it with the registry
    let mut create_and_register_package = |name: &str| -> Result<Id> {
        let pkg_path = dir_path.join(name);
        let package = Package::new(name, pkg_path)?;

        // Add a single commit adding a test file
        let mut file = File::create(package.path().join("patches/0001-test.patch")).unwrap();
        write!(&mut file, "{}", touch_file_patch(&format!("src/merlon_test_{name}"))).unwrap(); // TODO

        let id = registry.register(package)?;
        Ok(id)
    };

    // Create this dependency graph:
    //        Root      <-- We want to build this package 
    //      /     \
    //    DepA   DepB
    //      \     /
    //     SharedDep
    let root = create_and_register_package("Root")?;
    let dep_a = create_and_register_package("DepA")?;
    let dep_b = create_and_register_package("DepB")?;
    let shared_dep = create_and_register_package("SharedDep")?;
    dbg!(&root, &dep_a, &dep_b, &shared_dep);
    registry.add_direct_dependency(root, dep_a)?;
    registry.add_direct_dependency(root, dep_b)?;
    registry.add_direct_dependency(dep_a, shared_dep)?;

    // Initialise the root package and sync
    let root_package = registry.get_or_error(root)?.clone();
    let mut initialised = root_package.clone().to_initialised(InitialiseOptions {
        baserom: rom::baserom(),
        rev: Some(DECOMP_REV.to_string()),
    })?;
    initialised.set_registry(registry); // XXX
    initialised.sync_repo()?;
    initialised.update_patches_dir()?;

    // There should be 1 patch in the root package now
    let root_patches = initialised.package().path().join("patches");
    dbg!(root_patches
        .read_dir()?
        .map(|e| Ok(e?.file_name()))
        .collect::<Result<Vec<_>>>()?
    );
    assert_eq!(root_patches.read_dir()?.count(), 1);

    // If the patches applied correctly, all the test files should have been made
    assert!(initialised.subrepo_path().join("src/merlon_test_Root.c").is_file());
    assert!(initialised.subrepo_path().join("src/merlon_test_DepA.c").is_file());
    assert!(initialised.subrepo_path().join("src/merlon_test_DepB.c").is_file());
    assert!(initialised.subrepo_path().join("src/merlon_test_SharedDep.c").is_file());

    Ok(())
}

// Generate a random git-like commit hash
fn gen_random_commit_hash_for_patch() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut hash = String::with_capacity(40);
    for _ in 0..40 {
        hash.push(rng.gen_range(b'0'..b'9') as char);
    }
    hash
}

fn touch_file_patch(filename: &str) -> String {
    let hash = gen_random_commit_hash_for_patch();
    format!(r#"From {hash} Mon Sep 17 00:00:00 2001
From: Merlon test <merlontest@nanaian.town>
Date: Wed, 26 Apr 2023 22:40:19 +0100
Subject: test

---
    {filename}.c | 0
    1 file changed, 0 insertions(+), 0 deletions(-)
    create mode 100644 {filename}

diff --git a/{filename} b/{filename}
new file mode 100644
index 0000000..e69de29
-- 
2.39.0"#)
}