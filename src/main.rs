#![allow(dead_code)]

use std::io;
use std::fmt::Display;
type AttribSet = Vec<String>;

struct FDIter<'a> {
    fdlist: &'a FDList,
    index: usize
}

impl <'a> Iterator for FDIter <'a> {
    type Item = &'a FD;

    fn next(&mut self) -> Option<&'a FD> {
        let temp = self.fdlist.list.get(self.index);
        self.index += 1;

        temp
    }
}

#[derive(Debug, Clone)]
struct FDList {
    list: Vec<FD>
}

impl FDList {

    /// Construct a new list of functional dependencies
    fn new() -> Self {
        Self {
            list: vec![]
        }
    }

    /// Add a new functional dependency
    fn push(&mut self, fd: FD) {
        if !self.list.contains(&fd) {
            self.list.push(fd);
        }
    }

    /// Get all the other functional dependencies except the one provided
    fn all_except(&self, fd: &FD) -> FDList {
        let fd_list: Vec<FD> = self.list.iter().filter(|x| *x != fd).map(|x| x.to_owned()).collect();

        FDList {
            list: fd_list
        }
    }

    /// Return the closure of a set of attributes based on the
    /// current set of functional dependencies
    fn closure(&self, attribs: &AttribSet) -> AttribSet {
        let mut result: AttribSet = Vec::new();
        result.extend(attribs.to_owned());
        let mut closure_len: usize = result.len();

        loop {
            for fd in self.list.iter() {
                let mut satisfied: bool = true;
                for lh_element in &fd.det {
                    if !result.contains(lh_element) {
                        satisfied = false;
                    }
                }

                if satisfied {
                    let values: AttribSet = fd.dep.iter().filter(|x| !result.contains(x)).cloned().collect();
                    result.extend(values);
                }
            }

            if closure_len == result.len() {
                break;
            } else {
                closure_len = result.len();
            }
        }

        result
    }

    /// Find all the functional dependencies with the same determinant
    fn find_all(&self, determinant: &AttribSet) -> Vec<FD> {
        let mut isolated: Vec<FD> = Vec::new();
        for element in self.list.iter() {
            if element.det == *determinant {
                isolated.push(element.clone());
            }
        }

        isolated
    }

    fn iter(&self) -> FDIter {
        FDIter {
            fdlist: self,
            index: 0
        }
    }

    fn canonical_cover(&self) -> FDList {
        let mut result = FDList::new();
        for fd in self.list.iter() {
            result.push(fd.clone());
        }
        
        let mut decomposed = FDList::new();
        for fd in result.iter() {
            for decomp_fd in fd.decompose().list {
                decomposed.push(decomp_fd);
            }
        }
        result = decomposed;
        
        let mut changed = true;
        while changed {
            changed = false;
            let current_fds = result.list.clone();
            
            for fd in current_fds.iter() {
                if fd.det.len() <= 1 {
                    continue; // Skip if determinant is already minimal
                }
                
                for i in 0..fd.det.len() {
                    let mut reduced_det = fd.det.clone();
                    let _ = reduced_det.remove(i);
                    
                    let closure = result.closure(&reduced_det);
                    
                    if closure.contains(&fd.dep[0]) {
                        let new_fd = FD {
                            det: reduced_det,
                            dep: fd.dep.clone()
                        };
                        
                        result = result.all_except(fd);
                        result.push(new_fd);
                        
                        changed = true;
                        break;
                    }
                }
                
                if changed {
                    break;
                }
            }
        }
        
        let mut redundant_fds = Vec::new();
        let mut closure_compute = result.clone();
        
        for fd in result.list.iter() {
            let without_fd = closure_compute.all_except(fd);
            let without_fd_closure = without_fd.closure(&fd.det);
            
            if without_fd_closure.contains(&fd.dep[0]) {
                redundant_fds.push(fd.clone());
                closure_compute = closure_compute.all_except(fd);
            }
        }
        
        for redundant in redundant_fds {
            result = result.all_except(&redundant);
        }
        
        result.recompose()
    }

    fn recompose(&self) -> FDList {
        let mut changed = true;
        let mut result = self.clone();

        while changed {
            let mut temp = result.clone();
            changed = false;

            for fd in result.iter() {
                let all_with_same_det = result.iter().filter(|x| *x.det == fd.det).collect::<Vec<_>>();
                
                if all_with_same_det.len() > 1 { 

                    let mut composed = FD::new();
                    all_with_same_det[0].det.iter().for_each(|d| composed.add_determinant(d.to_string()));
                    all_with_same_det.iter().for_each(|f| {
                        f.dep.iter().for_each(|d| {
                            composed.add_dependant(d.to_string());
                        });
                    });

                    temp.list = temp.iter().filter(|f| {
                        f.det != composed.det
                    }).map(|x| x.to_owned()).collect::<Vec<_>>();

                    temp.push(composed);
                    changed = true;
                }
            }

            result = temp;
        }

        result
    }
}

#[derive(Debug, Clone)]
struct FD {
    det: AttribSet,
    dep: AttribSet
}

impl PartialEq for FD {
    fn eq(&self, other: &Self) -> bool {

        if self.det.len() != other.det.len() {
            return false;
        }

        if self.dep.len() != other.dep.len() {
            return false;
        }

        for attrib in self.det.iter() {
            if !other.det.contains(attrib) {
                return false;
            }
        }

        for attrib in self.dep.iter() {
            if !other.dep.contains(attrib) {
                return false;
            }
        }

        true
    }
}

impl Display for FD {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}->{}", self.det.join(", "), self.dep.join(", "))
    }
}


impl FD {
    fn new() -> FD {
        FD {
            det: vec![],
            dep: vec![]
        }
    }

    /// Check if the constructed FD is valid or not
    fn is_valid(&self) -> bool {
        if self.det.is_empty() || self.dep.is_empty() {
            return false;
        }

        true
    }

    /// Add to the add_determinant
    /// add_determinant("a") == "a" -> ""
    fn add_determinant(&mut self, attrib: String) {
        if !self.det.contains(&attrib) {
            self.det.push(attrib);
        }
    }

    /// Add to the add_dependant
    /// add_dependant("a") == "" -> "a"
    fn add_dependant(&mut self, attrib: String) {
        if !self.dep.contains(&attrib) {
            self.dep.push(attrib);
        }
    }

    /// Construct  a functional dependency from a piece of string data
    /// Expected String: "a,b"->"c,d"
    fn from(string: String) -> Result<FD, ()> {
        let mut result = FD {
            det: Vec::new(),
            dep: Vec::new()
        };

        if let Some((lefts, rights)) = string.split_once("->") {

            for element in lefts.split(",") {
                result.add_determinant(element.trim().to_string());
            }

            for element in rights.split(",") {
                result.add_dependant(element.trim().to_string());
            }
        }

        if !result.is_valid() {
            return Err(())
        }

        Ok(result)
    }

    /// Break down a functional dependency into several smaller ones if their
    /// dependents constitutes multiple attributes
    fn decompose(&self) -> FDList {
        let mut decomposed: FDList = FDList::new();

        for element in self.dep.clone() {
            decomposed.push(FD {
                det: self.det.to_owned(),
                dep: vec![
                    element
                ]
            });
        }

        decomposed
    }
}

fn main() {
    println!("Enter the functional dependencies: ");
    let mut fds: FDList = FDList::new();

    loop {
        let mut buffer: String = String::new();
        if io::stdin().read_line(&mut buffer).is_err() {
            continue;
        }

        if buffer.len() == 1 {
            break;
        }

        if let Ok(fd) = FD::from(buffer) {
            fds.push(fd);
        }
    }

    for fd in fds.canonical_cover().iter() {
        println!("{}", fd);
    }
}

