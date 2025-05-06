use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Error};

// pub struct TypeMap<T: Any> {
//     map: HashMap<TypeId, Box<dyn Any>>,
// }

// impl TypeMap {
//     pub fn new() -> Self {
//         Self {
//             map: HashMap::new(),
//         }
//     }

//     pub fn insert<T: Any + 'static>(&mut self, item: T) {
//         self.map.insert(TypeId::of::<T>(), Box::new(item));
//     }

//     pub fn get<T: Any + 'static>(&self) -> Option<&T> {
//         self.map
//             .get(&TypeId::of::<T>())
//             .map(|item| item.downcast_ref().expect("typemap internal error"))
//     }
// }

pub fn copy<U: AsRef<Path>, V: AsRef<Path>>(from: U, to: V) -> Result<(), Error> {
    let mut stack = Vec::new();
    stack.push(PathBuf::from(from.as_ref()));

    let output_root = PathBuf::from(to.as_ref());
    let input_root = PathBuf::from(from.as_ref()).components().count();

    while let Some(working_path) = stack.pop() {
        // Generate a relative path
        let src: PathBuf = working_path.components().skip(input_root).collect();

        // Create a destination if missing
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if fs::metadata(&dest).is_err() {
            fs::create_dir_all(&dest)
                .with_context(|| format!("could not create directory: {:?}", dest))?;
        }

        for entry in fs::read_dir(&working_path)
            .with_context(|| format!("could not read directory: {:?}", working_path))?
        {
            let entry = entry.with_context(|| {
                format!("could not read entry in directory: {:?}", working_path)
            })?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                match path.file_name() {
                    Some(filename) => {
                        let dest_path = dest.join(filename);
                        fs::copy(&path, &dest_path).with_context(|| {
                            format!("could not copy file from {:?} to {:?}", path, dest_path)
                        })?;
                    }
                    None => {
                        println!("failed: {:?}", path);
                    }
                }
            }
        }
    }

    Ok(())
}
