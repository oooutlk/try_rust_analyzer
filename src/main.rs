use home::{
    cargo_home,
    rustup_home,
};

use std::{
    env,
    path::Path,
};

use ra_ap_cfg::CfgDiff;

use ra_ap_ide::{
    FilePosition,
    TextSize,
};

use ra_ap_project_model::{
    CargoConfig,
    CargoWorkspace,
    CfgOverrides,
    ManifestPath,
    ProjectWorkspace,
    Sysroot,
    WorkspaceBuildScripts,
};

use ra_ap_rust_analyzer::cli::load_cargo;

use ra_ap_vfs::{
    AbsPathBuf,
    VfsPath,
};

fn main() {
    let mainfest_dir = env::var( "CARGO_MANIFEST_DIR" ).unwrap();
    let manifest_dir = Path::new( &mainfest_dir );

    let cargo_toml = ManifestPath::try_from(
        AbsPathBuf::assert(
            manifest_dir.join( "Cargo.toml" ).to_path_buf()
        )
    ).unwrap();

    let extra_env = ra_ap_ide_db::FxHashMap::default();

    fn do_nothing( _s: String ) {} 

    let cargo = CargoWorkspace::new(
        CargoWorkspace::fetch_metadata(
            &cargo_toml,
            &AbsPathBuf::assert( manifest_dir.to_path_buf() ),
            &CargoConfig::default(),
            &do_nothing
        ).unwrap()
    );

    do_nothing( String::default() );

    let toolchain = format!( "stable-{}", rustc_host::from_cli().unwrap() );

    let ws = ProjectWorkspace::Cargo {
        cargo,
        build_scripts: WorkspaceBuildScripts::default(),
        sysroot: Sysroot::load(
            AbsPathBuf::assert( rustup_home().unwrap().join("toolchains").join( toolchain )),
            AbsPathBuf::assert( cargo_home().unwrap().join("registry").join("src") )
        ).ok(),
        rustc: None,
        rustc_cfg: vec![],
        cfg_overrides: CfgOverrides::Wildcard( CfgDiff::new( vec![], vec![] ).unwrap() ),
        toolchain: None,
        target_layout: None,
    };

    let load_config = load_cargo::LoadCargoConfig {
        load_out_dirs_from_check : true,
        with_proc_macro : false,
        prefill_caches : false,
    };
    let (analysis_host, vfs, _proc_macro_server) =
        load_cargo::load_workspace( ws, &extra_env, &load_config ).unwrap();

    let vfs_path = VfsPath::new_real_path( manifest_dir.join("src").join("main.rs").to_string_lossy().to_string() );
    let file_id = vfs.file_id( &vfs_path ).unwrap();
    let offset = TextSize::try_from( 1070usize ).unwrap(); // refer to line 58, `do_nothing( String::default() );`
    let file_position = FilePosition{ file_id, offset };

    let analysis = analysis_host.analysis();
    let definition = analysis.goto_definition( file_position ).unwrap();
    println!( "{:?}", definition );
}
