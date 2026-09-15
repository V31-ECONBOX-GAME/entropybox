use econbox_art::{Icon, SPRITE, atlas, flatten, icon, png, sprite};
use econbox_creature::SPECIES;
use std::fs;
use std::path::PathBuf;

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|module| module.parent())
        .expect("workspace root")
        .join("assets/sprite");

    fs::create_dir_all(&root).expect("make assets/sprite");

    let frames: Vec<_> = SPECIES
        .iter()
        .map(|species| sprite(species.stance(), species.mark(), species.skin()))
        .collect();

    for (species, frame) in SPECIES.iter().zip(&frames) {
        let file = root.join(format!("{}.png", species.name()));
        fs::write(&file, png(SPRITE as u32, SPRITE as u32, &flatten(frame))).expect("write sprite");
    }

    let (data, width, height) = atlas(&frames);
    fs::write(root.join("creatures.png"), png(width, height, &data)).expect("write atlas");

    let kit: [(Icon, &str); 16] = [
        (Icon::Block, "block"),
        (Icon::Droplet, "droplet"),
        (Icon::Sprout, "sprout"),
        (Icon::Raise, "raise"),
        (Icon::Lower, "lower"),
        (Icon::Lens, "lens"),
        (Icon::Eraser, "eraser"),
        (Icon::Undo, "undo"),
        (Icon::Play, "play"),
        (Icon::Pause, "pause"),
        (Icon::Hourglass, "hourglass"),
        (Icon::Round, "round"),
        (Icon::Square, "square"),
        (Icon::Minus, "minus"),
        (Icon::Plus, "plus"),
        (Icon::Bucket, "bucket"),
    ];

    for (kind, name) in kit {
        let frame = icon(kind, None);
        fs::write(
            root.join(format!("icon-{name}.png")),
            png(SPRITE as u32, SPRITE as u32, &flatten(&frame)),
        )
        .expect("write icon");
    }

    println!(
        "{} creature sprites, {} tool icons and a {width}x{height} sheet in {}",
        frames.len(),
        kit.len(),
        root.display()
    );
}
