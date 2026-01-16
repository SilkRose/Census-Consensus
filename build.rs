use std::{ env, fs };
use std::io::{ self, Read as _, Write as _ };
use std::process::{ Command, Stdio };

fn main() {
	println!("cargo::rerun-if-changed=queries");
	println!("cargo::rerun-if-changed=migrations");
	println!("cargo::rerun-if-changed=src");
	println!("cargo::rerun-if-changed=target/unoptimised.css");

	let mut node_modules = env::current_dir().unwrap();
	node_modules.push("node_modules");

	let pnpm_path = run_command("which", ["pnpm"])();
	let pnpm_path = pnpm_path.trim();

	if !fs::exists(&*node_modules).unwrap() {
		let _ = run_command(pnpm_path, ["install"]);
	}

	let _ = run_command(pnpm_path, [
		"tailwindcss",
		"--input",
		"src/css.css",
		"--output",
		"target/unoptimised.css"
	]);

}

fn run_command<'h>(command: &str, args: impl IntoIterator<Item = &'h str> + Clone) -> impl FnOnce() -> String {
	let child = Command::new(command)
		.args(args.clone())
		.stdin(Stdio::null())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn();

	let whole_command = || {
		let mut whole_command = String::new();

		whole_command.push_str(command);
		whole_command.extend(args.into_iter().flat_map(|item| [" ", item]));

		whole_command
	};

	let mut child = match child {
		Ok(child) => { child }
		Err(err) => {
			eprintln!("running command wasn't successful");
			eprintln!("command: {}", whole_command());
			eprintln!("error: {err}");
			panic!()
		}
	};

	let mut stdout_stream = child.stdout.take().unwrap();
	let mut stderr_stream = child.stderr.take().unwrap();

	let result = child.wait().unwrap();
	let mut stdout = Vec::new();
	stdout_stream.read_to_end(&mut stdout).unwrap();

	if !result.success() {
		let mut stderr = Vec::new();
		stderr_stream.read_to_end(&mut stderr).unwrap();

		eprintln!("running command wasn't successful");
		eprintln!("command: {}", whole_command());

		eprintln!("--- command stdout\n");
		io::stderr().write_all(&stdout).unwrap();
		eprintln!();

		eprintln!("--- command stderr\n");
		io::stderr().write_all(&stderr).unwrap();
		eprintln!();

		panic!();
	}

	// this is so we don't validate stdout is valid utf-8
	// unless we _actually_ need it to be
	move || String::from_utf8(stdout).unwrap()
}
