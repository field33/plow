// // TODO: Just a sketch and notes, probably wont look like this..

// use camino::Utf8PathBuf;
// use reqwest::StatusCode;

// use crate::{
//     config::{create_configuration_directory_if_not_exists, PlowConfig},
//     error::CliError,
//     git::PublicIndexRepository,
//     subcommand::login::get_saved_api_token,
// };

// pub trait Sync {
//     fn sync() -> Result<(), CliError> {
//         let PlowConfig {
//             public_index_dir, ..
//         } = create_configuration_directory_if_not_exists()?;

//         let token = get_saved_api_token()?;

//         // Get properly registry url
//         let registry_url = "";
//         // Get private index cache
//         // Parse versions, create field hashes.
//         // Make request for diff

//         let mut private_index_sync_url = format!("{registry_url}/v1/index/private/sync");

//         let client = reqwest::blocking::Client::new();

//         let private_index_sync_response = client
//             .post(private_index_sync_url)
//             .header("Authorization", &format!("Basic {token}"))
//             .header("Content-Type", "application/json")
//             // Insert body with hashes
//             // .body(body)
//             .send()
//             // Request failed err..
//             .map_err(|err| CliError::Dummy)?;

//         let status = private_index_sync_response.status();

//         match status {
//             StatusCode::OK => {
//                 let response_body = private_index_sync_response
//                     .bytes()
//                     // Handle
//                     .map_err(|err| CliError::Dummy)?;

//                 // patch local private index cache with response body,
//                 // update local cache related files if necessary
//                 // continue with syncing public index..
//             }
//             StatusCode::INTERNAL_SERVER_ERROR => {
//                 // Give feedback and continue..
//             }
//             _ => {
//                 // Give feedback and continue..
//             }
//         }

//         // TODO: Change this to real index.
//         let clone_from = "git@github.com:field33/test-public-registry-index.git";

//         let repository = PublicIndexRepository::clone_or_open(
//             clone_from,
//             &public_index_dir,
//             "main",
//             Some(
//                 &Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR"))
//                     .join("test_public_registry_index_deploy_key"),
//             ),
//             None,
//         )
//         .unwrap();

//         repository.pull_from_origin_fast_forward().unwrap();

//         // Now that the repository is synced
//         // Get file hashes from the repo
//         // Compare with local cache
//         // If there are changes, patch the local cache file by..
//         // Only parsing the changed file hashes,
//         // replacing all versions in cache for those files
//         // writing back to cache
//         // updating necessary files

//         Ok(())
//     }
// }
