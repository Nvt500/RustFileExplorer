use std::fs;
use fs::read_dir;
use std::cell::RefCell;
use std::io::{Seek, Write};
use std::path::Path;
use std::rc::Rc;
use slint::{Model, ModelRc, SharedString};

mod dir;
use crate::dir::{Dir};

slint::include_modules!();


#[derive(Clone)]
pub struct Data
{
    pub array: Vec<SharedString>,
    pub index: usize,
    pub max_length: usize,
}


impl Data
{
    pub fn new(start: SharedString) -> Data
    {
        let mut v = Vec::new();
        v.push(start);
        Data { array: v, index: 0, max_length: 20, }
    }

    pub fn print(&self)
    {
        for a in &self.array
        {
            print!("{}, ", a);
        }
        println!("{:?}", self.index);
    }

    pub fn push(&mut self, shared_string: SharedString)
    {
        if self.index < self.array.len() - 1
        {
            for _ in 0..self.array.len() - 1 - self.index
            {
                self.array.pop();
            }
        }
        self.array.push(shared_string);
        if self.array.len() > self.max_length
        {
            self.array.remove(0);
        }
        else
        {
            self.index += 1;
        }
    }

    pub fn get(&mut self) -> SharedString
    {
        if self.index > 0
        {
            self.index -= 1;
            self.array[self.index].clone()
        }
        else
        {
            SharedString::new()
        }
    }

    pub fn regret(&mut self) -> SharedString
    {
        if self.index < self.array.len() - 1
        {
            self.index += 1;
            self.array[self.index ].clone()
        }
        else
        {
            SharedString::new()
        }
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    let app = Rc::new(ui);

    let parent = start_dir();
    let start = parent.path.clone();
    let start = start.to_str().unwrap();
    set_props(parent, &app);

    let actions = Rc::new(RefCell::new(Data::new(SharedString::from(start))));

    let app_clone = app.clone();
    let actions_clone = actions.clone();

    app.on_open_dir(move |path| {
        let p_path = Path::new(Path::new(path.as_str()));
        let dir = Dir::build(p_path);

        if dir.is_ok()
        {
            let mut data = actions_clone.borrow_mut();
            data.push(path.clone());

            set_props(dir.unwrap(), &app_clone);
        }
    });

    let app_clone = app.clone();

    app.on_refresh(move |path| {
        let path = Path::new(Path::new(path.as_str()));
        let dir = Dir::build(path);
        if dir.is_ok()
        {
            set_props(dir.unwrap(), &app_clone);
        }
    });

    let app_clone = app.clone();
    let actions_clone = actions.clone();

    app.on_back(move || {
        let mut data = actions_clone.borrow_mut();
        let path = data.get();
        if !path.is_empty()
        {
            let path = Path::new(Path::new(path.as_str()));
            let dir = Dir::build(path);
            if dir.is_ok()
            {
                set_props(dir.unwrap(), &app_clone);
            }
        }
    });

    let app_clone = app.clone();
    let actions_clone = actions.clone();

    app.on_forward(move || {
        let mut data = actions_clone.borrow_mut();
        let path = data.regret();
        if !path.is_empty()
        {
            let path = Path::new(Path::new(path.as_str()));
            let dir = Dir::build(path);
            if dir.is_ok()
            {
                set_props(dir.unwrap(), &app_clone);
            }
        }
    });

    let app_clone = app.clone();

    app.on_search(move |query, path| {
        if !query.is_empty()
        {
            let path = Path::new(Path::new(path.as_str()));
            let dir = Dir::build(path);
            if dir.is_ok()
            {
                let searches = dir.unwrap().search(query.to_lowercase().as_str());
                app_clone.set_children(ModelRc::from(searches.as_slice()));
            }
        }
    });

    app.on_new_file(move |path, file_clicked| {
        let path = Path::new(Path::new(path.as_str()));
        if file_clicked
        {
            match fs::File::create(path)
            {
                Ok(_) => {},
                Err(_) => {},
            }
        }
        else
        {
            match fs::create_dir_all(path)
            {
                Ok(_) => {},
                Err(_) => {},
            }
        }
    });

    app.on_delete_file(move |path| {
        let path = Path::new(Path::new(path.as_str()));
        match fs::remove_file(path)
        {
            Ok(_) => {},
            Err(_) => {
                match fs::remove_dir(path)
                {
                    Ok(_) => {},
                    Err(_) => {},
                }
            },
        }
    });

    app.on_rename_file(move |path, name| {
        let path = Path::new(Path::new(path.as_str()));
        let name = Path::new(Path::new(name.as_str()));
        let _ = fs::rename(path, name);
    });

    let app_clone = app.clone();

    app.on_save_file(move |path| {
        let path = Path::new(Path::new(path.as_str()));
        if let Ok(mut file) = fs::File::options().write(true).open(path)
        {
            file.set_len(0).unwrap();
            file.rewind().unwrap();
            file.write(app_clone.get_file().as_bytes()).unwrap();
        }
    });

    app.on_make_children_equal_FUCKING_ARRAY(move |dirs: ModelRc<Directory>| {
        let mut a = Vec::<SharedString>::new();
        for dir in dirs.iter()
        {
            a.push(dir.name);
        }
        ModelRc::from(a.as_slice())
    });

    app.run()
}


fn start_dir() -> Dir
{
    let dir = read_dir("./").unwrap();
    let dir = dir.last().unwrap().unwrap().path().canonicalize().unwrap();
    let dir: &Path = dir.as_os_str().to_str().unwrap()[4..].as_ref();
    let parent_dir = dir.parent().unwrap();
    Dir::build(parent_dir).unwrap()
}


fn set_props(dir: Dir, ui: &Rc<AppWindow>)
{
    if dir.dir_type.can_open()
    {
        set_for_dirs(dir, &ui);
    }
    else if dir.dir_type.can_read()
    {
        let mut content = dir.read();
        if content.is_empty()
        {
            content = "Empty File or Cannot Read".into();
        }
        ui.set_file(content.into());
        ui.set_children(ModelRc::default());
        let parents: Vec<Directory> = dir.get_parents()
                                         .unwrap()
                                         .iter()
                                         .map(|d| d.to_directory())
                                         .collect();
        let parents: ModelRc<Directory> = ModelRc::from(parents.as_slice());
        ui.set_parents(parents);
        ui.set_dir(dir.to_directory());
    }
}

fn set_for_dirs(dir: Dir, ui: &Rc<AppWindow>)
{
    if dir.dir_type.has_children()
    {
        let children = dir.get_children_as_directory();

        ui.set_children(ModelRc::from(children.as_slice()));
    }
    else if !dir.dir_type.can_open()
    {
        return;
    }
    else
    {
        ui.set_children(ModelRc::default());
    }

    if dir.dir_type.has_parent()
    {
        let parents: Vec<Directory> = dir.get_parents()
                                         .unwrap()
                                         .iter()
                                         .map(|d| d.to_directory())
                                         .collect();
        let parents: ModelRc<Directory> = ModelRc::from(parents.as_slice());

        ui.set_parents(parents);
    }
    else
    {
        ui.set_parents(ModelRc::default());
    }

    ui.set_dir(dir.to_directory());
    ui.set_file(SharedString::new());
}