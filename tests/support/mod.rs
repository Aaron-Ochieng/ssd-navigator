use std::fs;
use std::path::PathBuf;

use tempfile::TempDir;

pub fn write_temp_file(dir: &TempDir, name: &str, contents: &str) -> PathBuf {
    let path = dir.path().join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(&path, contents).expect("write fixture");
    path
}

#[allow(dead_code)]
pub fn write_requirements(dir: &TempDir, ids: &[&str]) -> PathBuf {
    let mut contents = String::from("requirements:\n");
    for id in ids {
        contents.push_str(&format!(
            "  - id: {id}\n    title: {id} title\n    description: \"desc\"\n",
        ));
    }
    write_temp_file(dir, "requirements.yaml", &contents)
}

#[allow(dead_code)]
pub fn write_tasks(dir: &TempDir, ids: &[&str]) -> PathBuf {
    let mut contents = String::from("tasks:\n");
    for (index, id) in ids.iter().enumerate() {
        contents.push_str(&format!(
            "  - id: TASK-{index:03}\n    requirementId: {id}\n    title: Task\n    status: open\n",
        ));
    }
    write_temp_file(dir, "tasks.yaml", &contents)
}
