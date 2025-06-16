use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Default)]
pub struct DirTree {
    base: PathBuf,
    tree: HashMap<PathBuf, Box<DirTree>>,
}

impl DirTree {
    pub fn mkdir_all<P: AsRef<Path>>(&mut self, dir: P) -> Result<()> {
        let mut elements = Vec::new();

        for component in dir.as_ref().components() {
            match component {
                Component::Normal(os_str) => elements.push(os_str),
                v => anyhow::bail!("invalid path component: {v:?}"),
            }
        }

        self.mkdir(&elements)
    }

    fn mkdir<P: AsRef<Path>>(&mut self, dir: &[P]) -> Result<()> {
        if dir.is_empty() {
            return Ok(());
        }

        let d = dir[0].as_ref();

        if let Some(child) = self.tree.get_mut(d) {
            return child.mkdir(&dir[1..]);
        }

        let full_dir = self.base.join(d);
        if !full_dir.exists() {
            fs::create_dir(&full_dir)?;
        }

        let mut child = Box::new(DirTree {
            base: full_dir,
            tree: HashMap::new(),
        });

        let res = child.mkdir(&dir[1..]);

        self.tree.insert(d.to_owned(), child);

        res
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mkdir_all() {
        let mut dt = DirTree::default();

        fs::remove_dir_all("foo").ok();

        {
            let dir = Path::new("foo/bar/baz");
            dt.mkdir_all(dir).unwrap();
            assert!(dir.exists());
        }

        {
            let dir = Path::new("foo/baz/fuz");
            dt.mkdir_all(dir).unwrap();
            assert!(dir.exists());
        }
    }
}
