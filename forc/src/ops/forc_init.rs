use crate::utils::defaults;
use std::fs;
use sway_utils::constants;

pub(crate) fn init_new_project(project_name: String) -> Result<(), Box<dyn std::error::Error>> {
    // Make a new directory for the project
    fs::create_dir_all(format!("{}/src", project_name))?;

    // Make directory for tests
    fs::create_dir_all(format!("{}/tests", project_name))?;

    // Insert default manifest file
    fs::write(
        format!("{}/{}", project_name, constants::MANIFEST_FILE_NAME),
        defaults::default_manifest(&project_name),
    )?;

    // Insert default test manifest file
    fs::write(
        format!("{}/{}", project_name, constants::TEST_MANIFEST_FILE_NAME),
        defaults::default_tests_manifest(&project_name),
    )?;

    // Insert default main function
    fs::write(
        format!("{}/src/main.sw", project_name),
        defaults::default_program(),
    )?;

    // Insert default test function
    fs::write(
        format!("{}/tests/harness.rs", project_name),
        defaults::default_test_program(),
    )?;

    Ok(())
}

#[test]
fn test() {
    //let example_url = "https://github.com/JoshuaBatty/sway/tree/master/examples/hello_world";
    let example_url = "https://github.com/JoshuaBatty/fuel_test_project";
    let my_project_name = "hello_world_test_root";

    let example_url = "https://github.com/JoshuaBatty/fuel_test_project/tree/main/hello_world_test";
    let my_project_name = "hello_world_test_one_deep";

    let example_url = "https://github.com/JoshuaBatty/fuel_test_project/tree/main/hello_world_test/subfolder/examples/hello_world";
    let my_project_name = "hello_world_test_very_deep";
    //let example_url = "https://github.com/ControlCplusControlV/Sushi-Sway";
    let (owner_name, project_name, examples_path) = parse_github_link(example_url).unwrap(); 

    let custom_url = format!("https://api.github.com/repos/{}/{}/contents/{}", owner_name, project_name, examples_path);
    println!("custom_url: {}", custom_url);

    // Get the path of the example we are using
    let path = std::env::current_dir().unwrap();
    
    let out_dir = path.join(my_project_name);
    let real_name = whoami::realname();

    match download_contents(&custom_url, &out_dir) {
        Ok(downloaded_dir) => println!("downloaded_dir: {}", downloaded_dir),
        Err(e) => println!("couldn't download from {}: {}", &custom_url, e),
    }

    // Change the project name and author of the Forc.toml file
    edit_forc_toml(&out_dir, my_project_name, &real_name).map_err(|e| eprintln!("{}", e)).ok();
    // Change the project name and authors of the Cargo.toml file
    edit_cargo_toml(&out_dir, my_project_name, &real_name).map_err(|e| eprintln!("{}", e)).ok();
}

use std::path::{Path, PathBuf};
use std::io::{Read, Write};
use std::fs::File;
use serde::{Deserialize};
use anyhow::Result;
use url::Url;

#[test]
fn test_parsing() {
    let example_url = "https://github.com/JoshuaBatty/sway/tree/master/examples/hello_world";
    let (owner_name, project_name, examples_path) = parse_github_link(example_url).unwrap();    
    assert_eq!(owner_name, "JoshuaBatty");
    assert_eq!(project_name, "sway");
    assert_eq!(examples_path, "examples/hello_world");


    let example_url = "https://github.com/FuelLabs/swayswap-demo/tree/master/contracts";
    let (owner_name, project_name, examples_path) = parse_github_link(example_url).unwrap();
    assert_eq!(owner_name, "FuelLabs");
    assert_eq!(project_name, "swayswap-demo");
    assert_eq!(examples_path, "contracts");
}

// #[test]
// fn test_github_url() {
//     let example_url = "https://github.com/JoshuaBatty/sway/tree/master/examples/hello_world";
//     let url = Url::parse(example_url).unwrap();
//     let github_url = is_parent_or_subfolder(&url);
//     assert_eq!(github_url, GithubUrl::SubDirectory);

//     let example_url = "https://github.com/FuelLabs/swayswap-demo/tree/master/contracts";
//     let url = Url::parse(example_url).unwrap();
//     let github_url = is_parent_or_subfolder(&url);
//     assert_eq!(github_url, GithubUrl::SubDirectory);

//     let example_url = "https://github.com/ControlCplusControlV/Sushi-Sway/";
//     let url = Url::parse(example_url).unwrap();
//     let github_url = is_parent_or_subfolder(&url);
//     assert_eq!(github_url, GithubUrl::Root);
// }

// #[derive(Debug, PartialEq)]
// enum GithubUrl {
//     Root,
//     SubDirectory,
// }

// fn is_parent_or_subfolder(url: &Url) -> GithubUrl {
//     if url.path().contains("tree") {
//         GithubUrl::SubDirectory
//     } else {
//         GithubUrl::Root
//     }
// }

fn parse_github_link(url: &str) -> Result<(String, String, String)> {
    let url = Url::parse(url)?;
    let mut path_segments = url.path_segments().ok_or_else(|| "cannot be base").unwrap();
    
    let owner_name = path_segments.next().unwrap();
    let project_name = path_segments.next().unwrap();

    // if path_segments.len() == 0 {
    //     return Ok((owner_name.to_string(), project_name.to_string(), None))
    // }

    match path_segments.skip(2)
        .map(|s| s.to_string())
        .reduce(|cur: String, nxt: String| format!("{}/{}", cur, nxt)) 
    {
        Some(examples_path) => Ok((owner_name.to_string(), project_name.to_string(), examples_path)),
        None => Ok((owner_name.to_string(), project_name.to_string(), "".to_string())),
    }
    // let examples_path = path_segments.skip(2)
    //     .map(|s| s.to_string())
    //     .reduce(|cur: String, nxt: String| format!("{}/{}", cur, nxt)).unwrap();

    // Ok((owner_name.to_string(), project_name.to_string(), Some(examples_path)))
}



fn edit_forc_toml(out_dir: &PathBuf, project_name: &str, real_name: &str) -> Result<()> {
    println!("forc path: {:#?}", out_dir.join(constants::MANIFEST_FILE_NAME));
    let mut file = File::open(out_dir.join(constants::MANIFEST_FILE_NAME))?;
    println!("DO WE GET HERE?");
    let mut toml = String::new();
    file.read_to_string(&mut toml)?;

    let mut manifest_toml = toml.parse::<toml_edit::Document>()?;
    manifest_toml["project"]["author"] = toml_edit::value(real_name);
    manifest_toml["project"]["name"] = toml_edit::value(project_name);
    
    let mut file = File::create(out_dir.join(constants::MANIFEST_FILE_NAME))?;
    file.write_all(manifest_toml.to_string().as_bytes())?;
    Ok(())
}

fn edit_cargo_toml(out_dir: &PathBuf, project_name: &str, real_name: &str) -> Result<()> {
    let mut file = File::open(out_dir.join(constants::TEST_MANIFEST_FILE_NAME))?;
    let mut toml = String::new();
    file.read_to_string(&mut toml)?;

    let mut manifest_toml = toml.parse::<toml_edit::Document>()?;
    manifest_toml["package"]["authors"] = toml_edit::value(format!("[{}]",real_name));
    manifest_toml["package"]["name"] = toml_edit::value(project_name);
    
    let mut file = File::create(out_dir.join(constants::TEST_MANIFEST_FILE_NAME))?;
    file.write_all(manifest_toml.to_string().as_bytes())?;
    Ok(())
}

fn download_file(url: &str, file_name: &str, out_dir: &Path) -> Result<String> {
    let mut data = Vec::new();
    let resp = ureq::get(url).call()?;
    resp.into_reader().read_to_end(&mut data)?;
    let path = out_dir.canonicalize()?.join(file_name);
    let mut file = File::create(&path)?;
    file.write_all(&data[..])?;
    Ok(path.to_str().unwrap().to_string())
}

pub fn download_contents(url: &str, out_dir: &Path) -> Result<String> {
    let responses: Vec<ContentResponse> = ureq::get(url).call()?.into_json()?;

    if !out_dir.exists() {
        fs::create_dir(out_dir)?;
    }

    responses.iter()
    .for_each(|response| {
        match &response.file_type {
            FileType::File => {
                if let Some(url) = &response.download_url {
                    download_file(url, &response.name, out_dir).unwrap();
                }
            }
            FileType::Dir => {
                match &response.name.as_str() {
                    // Only download the directory and its contents if it matches src or tests
                    &constants::SRC_DIR | &constants::TEST_DIRECTORY => {
                        println!("response.name {:?}", &response.name);
                        let dir = out_dir.join(&response.name);
                        let url = format!("{}/{}", url, response.name);
                        download_contents(&url, &dir).unwrap();
                    },
                    _ => ()
                }
            }
        }
    });

    // 1. Create a new directory for the project
    // 2. Parse the parent directory in a Vec<ContentResponse> type.
    // 3. for all file_type == "file" responses, download the file and save it to the project directory.
    // 4. for all file_type == "dir" responses, recursively call this function.

    Ok("".to_string())
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum FileType {
    File,
    Dir,
}
#[derive(serde::Deserialize, Debug)]
struct Links {
    git: String,
    html: String,
    #[serde(rename = "self")]
    cur: String,
}
#[derive(serde::Deserialize, Debug)]
struct ContentResponse {
    #[serde(rename = "_links")]
    links: Links,
    download_url: Option<String>,
    git_url: String,
    html_url: String,
    name: String,
    path: String,
    sha: String,
    size: u64,
    #[serde(rename = "type")]
    file_type: FileType,
    url: String,
}

pub (crate) fn init_counter_example(project_name: String) -> Result<(), Box<dyn std::error::Error>> {
    // Make a new directory for the project
    fs::create_dir_all(format!("{}/src", project_name))?;

    // Make directory for tests
    fs::create_dir_all(format!("{}/tests", project_name))?;

    // Get the path of the example we are using
    let path = std::env::current_dir()?;
    println!("The current directory is {}", path.display());

    unimplemented!();
}