use std::ffi::OsString;
use std::fs::read_dir;
use std::io::Read;
use std::path::{Path, PathBuf};
use crate::Directory;

pub enum DirType
{
    File,
    Directory,
    Empty,
    Root,
    None,
}

impl DirType
{
    pub fn has_parent(&self) -> bool
    {
        match self
        {
            DirType::File => true,
            DirType::Directory => true,
            DirType::Empty => true,
            DirType::Root => false,
            DirType::None => false,
        }
    }

    pub fn has_children(&self) -> bool
    {
        match self
        {
            DirType::File => false,
            DirType::Directory => true,
            DirType::Empty => false,
            DirType::Root => true,
            DirType::None => false,
        }
    }

    pub fn can_open(&self) -> bool
    {
        match self
        {
            DirType::File => false,
            DirType::Directory => true,
            DirType::Empty => true,
            DirType::Root => true,
            DirType::None => false,
        }
    }

    pub fn can_read(&self) -> bool
    {
        match self
        {
            DirType::File => true,
            DirType::Directory => false,
            DirType::Empty => false,
            DirType::Root => false,
            DirType::None => false,
        }
    }
}


pub struct Dir
{
    pub name: String,
    pub path: PathBuf,
    pub dir_type: DirType,
}


impl Dir
{
    pub fn build(path: &Path) -> Result<Dir, ()>
    {
        let mut dir = Dir { name: String::new(), path: PathBuf::from(path), dir_type: DirType::None };

        let name;
        match dir.path.file_name()
        {
            None => {
                if !dir.path.exists()
                {
                    return Err(())
                }

                name = dir.path.as_os_str();
            },
            Some(n) => name = n,
        };

        let name = name.to_str();

        if name.is_none()
        {
            return Err(());
        }
        dir.name = name.unwrap().to_string();

        if !dir.path.exists()
        {
            return Err(());
        }
        if dir.path.is_file()
        {
            dir.dir_type = DirType::File;
        }
        else
        {
            if dir.path.parent().is_none()
            {
                dir.dir_type = DirType::Root;
            }
            else if let Ok(d) = read_dir(&dir.path)
            {
                if d.count() == 0
                {
                    dir.dir_type = DirType::Empty;
                }
                else
                {
                    dir.dir_type = DirType::Directory;
                }
            }
            else
            {
                return Err(());
            }
        }

        Ok(dir)
    }

    pub fn get_parent(&self) -> Result<Dir, ()>
    {
        if !self.dir_type.has_parent()
        {
            return Err(());
        }

        Dir::build(self.path.parent().unwrap())
    }

    pub fn get_parents(&self) -> Result<Vec<Dir>, usize>
    {
        //  Usize is how many parents were processed before they couldn't be unwrapped.
        if !self.dir_type.has_parent()
        {
            return Err(0);
        }

        let mut dirs = Vec::<Dir>::new();
        let mut current = self;
        let mut index: usize = 0;

        loop
        {

            match current.path.parent()
            {
                None => return Err(index),
                Some(d) => {
                    let d = Dir::build(d);
                    if d.is_err()
                    {
                        return Err(index);
                    }
                    dirs.insert(0, d.unwrap());
                    current = &dirs[0];
                    index += 1;
                },
            }
            if !current.dir_type.has_parent()
            {
                return Ok(dirs);
            }
        }
    }

    pub fn get_children(&self) -> Result<Vec<Dir>, usize>
    {
        // Usize is how many children were processed before it failed to unwrap
        if !self.dir_type.has_children()
        {
            return Err(0);
        }

        let mut dirs = Vec::<Dir>::new();
        let mut index: usize = 0;

        let dir = read_dir(&self.path);
        if dir.is_err()
        {
            return Err(0);
        }

        for d in dir.unwrap()
        {
            match d
            {
                Ok(f) => {
                    match Dir::build(f.path().as_path())
                    {
                        Ok(d) => {
                            dirs.push(d);
                            index += 1;
                        },
                        Err(_) => return Err(index),
                    };
                },
                Err(_) => return Err(index),
            }
        }

        Ok(dirs)
    }

    pub fn get_children_ignore(&self) -> Vec<Dir>
    {
        if !self.dir_type.has_children()
        {
            return Vec::new();
        }

        let mut dirs = Vec::<Dir>::new();

        let dir = read_dir(&self.path);
        if dir.is_err()
        {
            return Vec::new();
        }

        for d in dir.unwrap()
        {
            match d
            {
                Ok(f) => {
                    match Dir::build(f.path().as_path())
                    {
                        Ok(d) => {
                            dirs.push(d);
                        },
                        Err(_) => {},
                    };
                },
                Err(_) => {},
            }
        }

        dirs
    }

    pub fn get_children_as_directory(&self) -> Vec<Directory>
    {
        if !self.dir_type.has_children()
        {
            return Vec::new();
        }

        let mut dirs = Vec::<Directory>::new();
        let dir = read_dir(&self.path);
        if dir.is_err()
        {
            return Vec::new();
        }

        for entry in dir.unwrap()
        {
            if let Ok(path) = entry
            {
                dirs.push(Dir::make_directory( path.file_name().to_str().unwrap(), path.path() ));
            }

        }

        dirs
    }

    pub fn get_children_as_name(&self) -> Vec<OsString>
    {
        if !self.dir_type.has_children()
        {
            return Vec::new();
        }

        let mut dirs = Vec::<OsString>::new();

        let dir = read_dir(&self.path);
        if dir.is_err()
        {
            return Vec::new();
        }

        for entry in dir.unwrap()
        {
            if let Ok(path) = entry
            {
                dirs.push(path.file_name());
            }
        }

        dirs
    }

    pub fn to_directory(&self) -> Directory
    {
        let mut d = Directory::default();
        d.name = (&self.name).into();
        d.path = self.path.to_str().unwrap().into();
        d
    }

    pub fn make_directory(name: &str, path: PathBuf) -> Directory
    {
        let mut d = Directory::default();
        d.name = name.into();
        d.path = path.to_str().unwrap().into();
        d
    }

    pub fn search(&self, query: &str) -> Vec<Directory>
    {
        if !self.dir_type.has_children()
        {
            return Vec::new();
        }

        let mut dirs = Vec::<Directory>::new();
        for child in self.get_children_ignore()
        {
            if child.name.to_lowercase().contains(query)
            {
                dirs.push(child.to_directory());
            }
            if child.dir_type.has_children()
            {
                dirs.extend(child.search(&query));
            }
        }
        dirs
    }

    pub fn read(&self) -> String
    {
        let file = std::fs::File::open(&self.path);

        if file.is_err()
        {
            return String::new();
        }

        let mut content: [u8; 10000] = ['\n' as u8; 10000];

        match file.unwrap().read(&mut content)
        {
            Ok(c) => {
                let mut s = String::from_utf8_lossy(&content).to_string();
                if c < content.len()
                {
                    s = s.trim_end().to_string();
                }
                s
            },
            Err(_) => String::new(),
        }
    }
}