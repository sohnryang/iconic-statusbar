extern crate i3ipc;

use std::{collections::HashMap, sync::mpsc, thread};

use i3ipc::{
    event::{inner::WindowChange, Event},
    reply::NodeType,
    I3Connection, I3EventListener, Subscription,
};
use iconic_statusbar::{tree_util::filter_childs_by_type, x11_util::wm_class_from_window_id};

#[derive(Debug)]
enum WindowType {
    Console,
    Firefox,
    Spotify,
    Discord,
    GenericWindow,
}

enum Message {
    UpdateWindowTable(HashMap<i32, Vec<WindowType>>),
    UpdateWorkspaceFocus(i32),
}

fn main() {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut connection = I3Connection::connect().unwrap();

        fn classify_window(class: (String, String)) -> WindowType {
            if class.1 == "kitty".to_owned() {
                WindowType::Console
            } else if class.0 == "spotify".to_owned() {
                WindowType::Spotify
            } else if class.1 == "firefox".to_owned() {
                WindowType::Firefox
            } else if class.1 == "discord".to_owned() {
                WindowType::Discord
            } else {
                WindowType::GenericWindow
            }
        }

        fn create_window_table(connection: &mut I3Connection) -> HashMap<i32, Vec<WindowType>> {
            let tree_root = connection.get_tree().unwrap();
            let workspaces = filter_childs_by_type(tree_root, NodeType::Workspace);

            let mut window_node_table = HashMap::new();
            for workspace in workspaces {
                let workspace_id = match workspace.clone().name.unwrap_or_default().parse::<i32>() {
                    Ok(x) => x,
                    Err(_) => continue,
                };
                let child_windows = filter_childs_by_type(workspace, NodeType::Con);
                window_node_table.insert(workspace_id, child_windows);
            }
            let mut window_table = HashMap::new();
            for (workspace_id, window_nodes) in window_node_table {
                let classes = window_nodes
                    .iter()
                    .map(|x| wm_class_from_window_id(x.window.unwrap_or_default() as u32))
                    .filter(|x| x.is_some())
                    .map(|x| classify_window(x.unwrap()))
                    .collect::<Vec<_>>();
                window_table.insert(workspace_id, classes);
            }
            window_table
        }
        tx.send(Message::UpdateWindowTable(create_window_table(
            &mut connection,
        )))
        .unwrap();

        let mut listener = I3EventListener::connect().unwrap();
        let subs = [Subscription::Workspace, Subscription::Window];
        listener.subscribe(&subs).ok();

        for event in listener.listen() {
            match event.unwrap() {
                Event::WorkspaceEvent(e) => {
                    let current_workspace_id =
                        match e.current.unwrap().name.unwrap_or_default().parse::<i32>() {
                            Ok(x) => x,
                            Err(_) => continue,
                        };
                    tx.send(Message::UpdateWorkspaceFocus(current_workspace_id))
                        .unwrap();
                }
                Event::WindowEvent(e) => {
                    match e.change {
                        WindowChange::New | WindowChange::Close | WindowChange::Move => (),
                        _ => continue,
                    };
                    tx.send(Message::UpdateWindowTable(create_window_table(
                        &mut connection,
                    )))
                    .unwrap();
                }
                _ => unreachable!(),
            }
        }
    });

    for received in rx {
        match received {
            Message::UpdateWindowTable(window_table) => {
                println!("window table: {:?}", window_table)
            }
            Message::UpdateWorkspaceFocus(workspace_id) => {
                println!("focused workspace: {}", workspace_id)
            }
        };
    }
}
