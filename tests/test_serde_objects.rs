use serde::{Deserialize, Serialize};
use std::thread;
use std::thread::JoinHandle;
use std::time;

mod helpers;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
struct LargeObject {
    field: String,
    fields: Vec<String>,
}

#[test]
fn test_serde_objects() {
    let mcrouter = helpers::connect("memcache://localhost:11311?protocol=ascii").unwrap();

    let key = "large_obj";

    for i in 0..20 {
        let object = LargeObject {
            field: format!("{}", i + 29).repeat(2000),
            fields: vec![format!("x{}>{}", i, i); 2002],
        };

        let got = mcrouter.set(key, object.clone(), time::Duration::from_secs(0));
        assert!(got.is_ok());

        let value: Option<LargeObject> = mcrouter.get(key).unwrap();
        assert_eq!(value.as_ref(), Some(&object));
    }
}

#[test]
fn test_serde_objects_with_pool() {
    let mcrouter = helpers::connect("memcache://localhost:11311?protocol=ascii").unwrap();

    let key = "large_obj_threaded";

    let total_threads = 40;

    let mut handles: Vec<Option<JoinHandle<_>>> = Vec::new();
    for x in 0..total_threads {
        let mcrouter_clone = mcrouter.clone();

        handles.push(Some(thread::spawn(move || {
            for y in 0..total_threads * 3 {
                let object = LargeObject {
                    field: format!("{}", y + 29).repeat(y),
                    fields: vec![format!("x: {} y: {}", x, y); x],
                };

                let got = mcrouter_clone.set(key, object.clone(), time::Duration::from_secs(5));
                assert!(got.is_ok());

                let value: Option<LargeObject> = mcrouter_clone.get(key).unwrap();
                assert!(value.is_some());
            }
        })));
    }

    for i in 0..total_threads {
        handles[i].take().unwrap().join().unwrap();
    }
}
