use std::{
    collections::HashSet,
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{self, Command},
};

pub mod args;
pub mod fs_save;
pub mod imkv;
pub mod save_stubs;

use {
    args::{Cli, Command::*, GlobalArgs},
    fs_save::*,
    imkv::{Imen, Imkv},
};

use binrw::{BinRead, BinWrite};
use clap::Parser;

const FS_INDEX_SAVEID: ProgramId = HexU64(0x8000000000000000);

fn print_imkvdb(_global: GlobalArgs, file: PathBuf, filter_ids: Option<String>) {
    let mut fp = fs::File::open(file).unwrap();
    let imkv = imkv::Imkv::read(&mut fp).unwrap();

    let filter_ids: HashSet<u64> = filter_ids
        .into_iter()
        .flat_map(|id| u64::from_str_radix(id.strip_prefix("0x").unwrap_or(&id), 16))
        .collect();

    for imen in imkv.entries() {
        let key = imen.key();
        let attr = SaveDataAttribute::read(&mut io::Cursor::new(key)).unwrap();

        if !filter_ids.is_empty() && !filter_ids.contains(&attr.system_save_data_id) {
            continue;
        }

        let value = imen.value();
        let val = WhateverTheValueIs::read(&mut io::Cursor::new(value)).unwrap();

        println!("{imen}\n  Key: {attr:?}\n  Value: {val:?}");
    }
}

fn extend_imkvdb_from_saves_list(
    imkv: &mut imkv::Imkv,
    saves: impl Iterator<Item = impl AsRef<Path>>,
) {
    for save in saves {
        let save = save.as_ref();
        let p = PathBuf::from(save);
        let sav = fs::read(&p).unwrap();

        //FIXME: dont depend on the savename being correct, parse its header instead
        let name = p.file_name().unwrap();
        let saveid = u64::from_str_radix(name.to_str().unwrap(), 16).unwrap();

        if saveid == FS_INDEX_SAVEID.0 || !HexU64(saveid).to_string().starts_with('8') {
            continue;
        }

        let key = SaveDataAttribute::new_system(saveid);
        let val = WhateverTheValueIs {
            saveid: saveid.into(),
            save_size: sav.len() as _,
            unknown: [0; 0x30],
        };

        println!("{save:?}:");

        let mut key_raw = Vec::with_capacity(0x40);
        key.write(&mut io::Cursor::new(&mut key_raw)).unwrap();
        let mut value_raw = Vec::with_capacity(0x40);
        val.write(&mut io::Cursor::new(&mut value_raw)).unwrap();

        if imkv.entries().iter().any(|i| i.key() == &key_raw) {
            println!(
                "Skipped registering {} with IMKV because there already is an entry present",
                HexU64(saveid)
            );
            continue;
        }

        let imen = Imen::from_kv(key_raw, value_raw);
        println!("\t{imen}");

        imkv.map_entries(|e| e.push(imen));
    }
}

fn tempdir(purpose: &str) -> PathBuf {
    dbg!(env::temp_dir().join(format!("savegen.{}.{}", purpose, process::id())))
}

fn make_save_from_imkv(
    hactoolnet: Option<PathBuf>,
    saveid: impl Into<ProgramId>,
    imkv: &Imkv,
    tmpdir: Option<impl Into<PathBuf>>,
    outdir: impl AsRef<Path>,
    outdir_is_file: bool,
) -> io::Result<PathBuf> {
    let tmpdir = match tmpdir {
        Some(dir) => dir.into(),
        None => tempdir("gen_save"),
    };
    fs::remove_dir_all(&tmpdir).ok();
    fs::create_dir_all(&tmpdir)?;

    {
        let imkv_path = tmpdir.join("imkvdb.arc");
        let mut fp = fs::File::create(&imkv_path)?;
        imkv.write(&mut fp).unwrap();
        println!("Wrote {imkv_path:?}");

        let id_path = tmpdir.join("lastPublishedId");
        let mut fp = fs::File::create(&id_path)?;
        fp.write_all(&0u64.to_le_bytes())?;
        println!("Wrote {id_path:?}");
    }
    let save_path = if outdir_is_file {
        outdir.as_ref().to_owned()
    } else {
        let save_path = outdir.as_ref().join(saveid.into().to_string());
        let sav = save_stubs::Savefile::new(5, FS_INDEX_SAVEID);

        let mut fp = fs::File::create(&save_path).unwrap();
        fp.write(sav.as_ref()).unwrap();
        save_path
    };

    Command::new(hactoolnet.unwrap_or("hactoolnet".into()))
        .arg(&save_path)
        .args(["--disablekeywarns", "-t", "save", "--sign", "--repack",])
        .arg(&tmpdir)
    .output()
    .unwrap();
    
    println!("Packed {tmpdir:?} into {save_path:?}");
    Ok(save_path)
}

fn gen_save(global: GlobalArgs, outdir: PathBuf, saves: Vec<PathBuf>) -> io::Result<()> {
    let saves: HashSet<PathBuf> = HashSet::from_iter(saves.into_iter());

    let mut imkv = Imkv::default();
    extend_imkvdb_from_saves_list(&mut imkv, saves.iter());

    make_save_from_imkv(global.hactoolnet, FS_INDEX_SAVEID, &imkv, global.tmpdir, outdir, false)?;
    Ok(())
}

fn update_save(global: GlobalArgs, save_path: PathBuf, saves: Vec<PathBuf>) -> io::Result<()> {
    let tmpdir = match global.tmpdir {
        Some(ref dir) => dir.into(),
        None => tempdir("extract-save"),
    };
    fs::create_dir_all(&tmpdir).unwrap();

    std::process::Command::new(global.hactoolnet.clone().unwrap_or("hactoolnet".into()))
        .arg(&save_path)
        .args(["--disablekeywarns", "-t", "save", "--outdir"])
        .arg(&tmpdir)
        .output()
        .unwrap();

    println!("{tmpdir:?}");
    let mut fp = fs::File::open(tmpdir.join("imkvdb.arc")).unwrap();
    let mut imkv = Imkv::read(&mut fp).unwrap();
    let saves: HashSet<PathBuf> = HashSet::from_iter(saves.into_iter());
    extend_imkvdb_from_saves_list(&mut imkv, saves.into_iter());

    make_save_from_imkv(global.hactoolnet, FS_INDEX_SAVEID, &imkv, global.tmpdir, save_path, true).unwrap();

    Ok(())
}

fn gen_for_mount(global: GlobalArgs, sysmount: PathBuf) -> io::Result<()> {
    let save_path = sysmount.join("save").join(FS_INDEX_SAVEID.to_string());
    let saves = fs::read_dir(sysmount.join("save"))?
        .into_iter()
        .flat_map(|i| i.map(|e| e.path()))
        .collect::<Vec<_>>();
    println!("{saves:?}");
    update_save(global, save_path, saves)
}

fn main() -> io::Result<()> {
    let args = Cli::parse();
    match args.command {
        Print { filter_ids, file } => print_imkvdb(args.global, file, filter_ids),
        GenSave { outdir, saves } => gen_save(args.global, outdir, saves)?,
        UpdateSave { save, saves } => update_save(args.global, save, saves)?,
        FixSys { sysmount } => gen_for_mount(args.global, sysmount).unwrap(),
    }

    Ok(())
}
