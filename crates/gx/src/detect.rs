use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectType {
    TypescriptBun,
    TypescriptNode,
    Rust,
    Go,
    Python,
    Generic,
}

impl ProjectType {
    pub fn as_str(self) -> &'static str {
        match self {
            ProjectType::TypescriptBun => "typescript-bun",
            ProjectType::TypescriptNode => "typescript-node",
            ProjectType::Rust => "rust",
            ProjectType::Go => "go",
            ProjectType::Python => "python",
            ProjectType::Generic => "generic",
        }
    }

    pub fn from_str_opt(s: &str) -> Option<Self> {
        Some(match s {
            "typescript-bun" => ProjectType::TypescriptBun,
            "typescript-node" => ProjectType::TypescriptNode,
            "rust" => ProjectType::Rust,
            "go" => ProjectType::Go,
            "python" => ProjectType::Python,
            "generic" => ProjectType::Generic,
            _ => return None,
        })
    }
}

/// Detect project type by checking for manifest files in `dir`.
/// Detection order mirrors `src/lib/detect.ts`:
/// package.json+bun.lock[b] > package.json > Cargo.toml > go.mod >
/// pyproject.toml/requirements.txt > generic.
pub fn detect_project_type(dir: &Path) -> ProjectType {
    let has = |name: &str| dir.join(name).exists();

    if has("package.json") {
        if has("bun.lock") || has("bun.lockb") {
            return ProjectType::TypescriptBun;
        }
        return ProjectType::TypescriptNode;
    }
    if has("Cargo.toml") {
        return ProjectType::Rust;
    }
    if has("go.mod") {
        return ProjectType::Go;
    }
    if has("pyproject.toml") || has("requirements.txt") {
        return ProjectType::Python;
    }
    ProjectType::Generic
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn touch(dir: &Path, name: &str) {
        fs::write(dir.join(name), "").unwrap();
    }

    #[test]
    fn typescript_bun_with_bun_lock() {
        let tmp = TempDir::new().unwrap();
        touch(tmp.path(), "package.json");
        touch(tmp.path(), "bun.lock");
        assert_eq!(detect_project_type(tmp.path()), ProjectType::TypescriptBun);
    }

    #[test]
    fn typescript_bun_with_bun_lockb() {
        let tmp = TempDir::new().unwrap();
        touch(tmp.path(), "package.json");
        touch(tmp.path(), "bun.lockb");
        assert_eq!(detect_project_type(tmp.path()), ProjectType::TypescriptBun);
    }

    #[test]
    fn typescript_node_with_only_package_json() {
        let tmp = TempDir::new().unwrap();
        touch(tmp.path(), "package.json");
        assert_eq!(detect_project_type(tmp.path()), ProjectType::TypescriptNode);
    }

    #[test]
    fn rust_with_cargo_toml() {
        let tmp = TempDir::new().unwrap();
        touch(tmp.path(), "Cargo.toml");
        assert_eq!(detect_project_type(tmp.path()), ProjectType::Rust);
    }

    #[test]
    fn go_with_go_mod() {
        let tmp = TempDir::new().unwrap();
        touch(tmp.path(), "go.mod");
        assert_eq!(detect_project_type(tmp.path()), ProjectType::Go);
    }

    #[test]
    fn python_with_pyproject() {
        let tmp = TempDir::new().unwrap();
        touch(tmp.path(), "pyproject.toml");
        assert_eq!(detect_project_type(tmp.path()), ProjectType::Python);
    }

    #[test]
    fn python_with_requirements_txt() {
        let tmp = TempDir::new().unwrap();
        touch(tmp.path(), "requirements.txt");
        assert_eq!(detect_project_type(tmp.path()), ProjectType::Python);
    }

    #[test]
    fn generic_when_no_manifest() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(detect_project_type(tmp.path()), ProjectType::Generic);
    }

    #[test]
    fn typescript_bun_beats_rust() {
        let tmp = TempDir::new().unwrap();
        touch(tmp.path(), "package.json");
        touch(tmp.path(), "bun.lock");
        touch(tmp.path(), "Cargo.toml");
        assert_eq!(detect_project_type(tmp.path()), ProjectType::TypescriptBun);
    }

    #[test]
    fn rust_beats_go() {
        let tmp = TempDir::new().unwrap();
        touch(tmp.path(), "Cargo.toml");
        touch(tmp.path(), "go.mod");
        assert_eq!(detect_project_type(tmp.path()), ProjectType::Rust);
    }
}
