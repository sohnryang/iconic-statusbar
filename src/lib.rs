pub mod tree_util {
    use i3ipc::reply::{Node, NodeType};

    pub fn filter_childs_by_type(root: Node, nodetype: NodeType) -> Vec<Node> {
        let mut stack = vec![root];
        let mut filtered_children = vec![];
        while !stack.is_empty() {
            let node = stack.pop().unwrap();
            if node.nodetype == nodetype {
                filtered_children.push(node.clone());
            }
            for child in node.nodes {
                stack.push(child);
            }
        }
        filtered_children
    }
}

pub mod x11_util {
    use std::process::Command;

    pub fn parse_xprop_wm_class(output: Vec<u8>) -> Option<(String, String)> {
        let encoded = String::from_utf8(output)
            .unwrap_or_default()
            .trim()
            .to_owned();
        let quote_pair = match encoded.strip_prefix("WM_CLASS = ") {
            Some(x) => x,
            None => return None,
        };
        let splitted = quote_pair.split(", ").collect::<Vec<_>>();
        if splitted.len() != 2 {
            return None;
        }
        let res_name = match splitted[0]
            .strip_prefix("\"")
            .unwrap_or_default()
            .strip_suffix("\"")
        {
            Some(x) => x,
            None => return None,
        };
        let res_class = match splitted[1]
            .strip_prefix("\"")
            .unwrap_or_default()
            .strip_suffix("\"")
        {
            Some(x) => x,
            None => return None,
        };
        return Some((res_name.to_string(), res_class.to_string()));
    }

    pub fn wm_class_from_window_id(window_id: u32) -> Option<(String, String)> {
        // TODO: use an X11 library instead of running a command and parsing its output.
        let output = match Command::new("xprop")
            .arg("-notype")
            .arg("-id")
            .arg(window_id.to_string())
            .arg("WM_CLASS")
            .output()
        {
            Ok(x) => x,
            Err(_) => return None,
        };
        if !output.status.success() {
            return None;
        }
        return parse_xprop_wm_class(output.stdout);
    }

    mod tests {
        #[test]
        fn test_parse_xprop_wm_class() {
            use super::parse_xprop_wm_class;
            assert_eq!(parse_xprop_wm_class(vec![0, 1, 2, 3, 4]), None);
            assert_eq!(parse_xprop_wm_class(vec![0, 0, 0, 0]), None);
            assert_eq!(
                parse_xprop_wm_class("hell world!".as_bytes().to_owned()),
                None
            );
            assert_eq!(
                parse_xprop_wm_class("WM_CLASS:  not found.".as_bytes().to_owned()),
                None
            );
            assert_eq!(
                parse_xprop_wm_class(
                    "WM_CLASS = \"Navigator\", \"firefox\""
                        .as_bytes()
                        .to_owned()
                ),
                Some(("Navigator".to_owned(), "firefox".to_owned()))
            );
        }
    }
}
