// // crates/selfie/tests/package_repository_tests.rs
// use selfie::{
//     fs::filesystem::MockFileSystem,
//     package::{port::PackageRepository, repository::YamlPackageRepository},
// };
// use std::path::{Path, PathBuf};
//
// #[test]
// fn test_repository_with_mixed_package_formats() {
//     let mut fs = MockFileSystem::default();
//     let package_dir = PathBuf::from("/test/packages");
//
//     // Mock filesystem with packages in different formats
//     fs.expect_path_exists()
//         .with(mockall::predicate::eq(package_dir.clone()))
//         .returning(|_| true);
//
//     // Mock directory listing with different file types
//     fs.mock_list_directory(
//         package_dir.clone(),
//         &[
//             package_dir.join("package1.yaml"),
//             package_dir.join("package2.yml"),
//             package_dir.join("not-a-package.txt"),
//             package_dir.join("README.md"),
//         ],
//     );
//
//     // Mock valid packages
//     let package1_yaml = r#"
// name: package1
// version: 1.0.0
// environments:
//   test-env:
//     install: echo "installing package1"
// "#;
//
//     let package2_yaml = r#"
// name: package2
// version: 2.0.0
// environments:
//   test-env:
//     install: echo "installing package2"
// "#;
//
//     fs.mock_read_file(package_dir.join("package1.yaml"), package1_yaml);
//     fs.mock_read_file(package_dir.join("package2.yml"), package2_yaml);
//
//     // Set up path existence checks
//     for file in ["package1.yaml", "package2.yml"] {
//         fs.mock_path_exists(package_dir.join(file), true);
//     }
//
//     let repo = YamlPackageRepository::new(fs, &package_dir);
//     let result = repo.list_packages().unwrap();
//
//     // Should find exactly two valid packages
//     assert_eq!(result.valid_packages().count(), 2);
//
//     // Verify package names
//     let names: Vec<&str> = result.valid_packages().map(|p| p.name()).collect();
//     assert!(names.contains(&"package1"));
//     assert!(names.contains(&"package2"));
// }
//
// #[test]
// fn test_repository_duplicate_package_names() {
//     let mut fs = MockFileSystem::default();
//     let package_dir = PathBuf::from("/test/packages");
//
//     // Mock filesystem with a package name that exists in both .yaml and .yml formats
//     fs.expect_path_exists()
//         .with(mockall::predicate::eq(package_dir.clone()))
//         .returning(|_| true);
//
//     // Set up path existence checks for duplicate package
//     fs.mock_path_exists(package_dir.join("duplicate.yaml"), true);
//     fs.mock_path_exists(package_dir.join("duplicate.yml"), true);
//
//     let repo = YamlPackageRepository::new(fs, &package_dir);
//     let result = repo.get_package("duplicate");
//
//     // Should return an error about multiple packages
//     assert!(matches!(
//         result,
//         Err(selfie::package::port::PackageRepoError::MultiplePackagesFound(_))
//     ));
// }
