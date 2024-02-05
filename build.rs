
fn main() {
    let time_mins = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()/60;

    let cargo_toml = std::fs::read_to_string("Cargo.toml").unwrap();



    let computers_dir = std::path::Path::new("./dist");

    let computer_builds = std::fs::read_dir(computers_dir).unwrap();

    let computer_builds = computer_builds
        .map(|x| x.unwrap().path())
        .filter(|x| !x.is_dir())
        .map(|x| x.file_name().unwrap().to_str().unwrap().to_string().split('.')
            .next().unwrap().to_string())
        .collect::<Vec<String>>();
    
    let mut computer_builds_versions = computer_builds
        .iter()
        .map(|x| {
            x.split("_").last().unwrap().parse::<u32>().unwrap()
        })
        .collect::<Vec<u32>>();

    computer_builds_versions.sort();

    let latest_version = computer_builds_versions.last().unwrap_or(&0);

    let new_version = latest_version+1;

    let mut lines: Vec<String> = cargo_toml.lines().map(|x| x.to_string()).collect();
    let lib_pos = lines.iter().position(|x| x.contains("[lib]")).unwrap();
    let name_pos = lines.iter().skip(lib_pos).position(|x| x.starts_with("name")).unwrap();

    lines[lib_pos+name_pos] = format!("name = \"computer_{}\"", new_version);

    let new_cargo_toml = lines.join("\n");

    std::fs::write("Cargo.toml", new_cargo_toml).unwrap();
}