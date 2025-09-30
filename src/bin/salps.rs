
use goreutils::args;

#[derive(Debug)]
enum ConfigWhich {
	Word,
	Sentence,
	Paragraph,
}

#[derive(Debug)]
struct Config {
	help: bool,
	version: bool,
	word_len: Option<(u32, Option<u32>)>,
	sentence_len: Option<(u32, Option<u32>)>,
	paragraph_len: Option<(u32, Option<u32>)>,
	which: ConfigWhich,
	seed: Option<u64>,
}
impl Default for Config {
	fn default() -> Self {
		Self {
			help: false,
			version: false,
			word_len: None,
			sentence_len: None,
			paragraph_len: None,
			which: ConfigWhich::Paragraph,
			seed: None,
		}
	}
}

const RULES: &[args::Rule<Config>] = &[
	("help", None, &|c, _, _| {
		c.help = true;
		Ok(())
	}),
	("version", None, &|c, _, _| {
		c.version = true;
		Ok(())
	}),
	("seed", None, &|c, a, e| {
		let Ok(x) = a() else {
			write!(e, "word: missing parameter").map_err(|_| ())?;
			return Err(());
		};

		if x == "_" {
			c.seed = None;
			return Ok(());
		}

		let Ok(x) = u64::from_str_radix(x, 10) else {
			write!(e, "word: unparsable input").map_err(|_| ())?;
			return Err(());
		};
		
		c.seed = Some(x);

		Ok(())
	}),
	("word", Some('w'), &|c, a, e| {
		let Ok(arg) = a() else {
			write!(e, "word: missing parameter").map_err(|_| ())?;
			return Err(());
		};

		c.which = ConfigWhich::Word;

		let mut split = arg.split(":");

		let Some(x) = split.next() else {
			unreachable!();
		};

		if x == "_" {
			c.word_len = None;
			return Ok(());
		}

		let Ok(x) = u32::from_str_radix(x, 10) else {
			write!(e, "word: unparsable input").map_err(|_| ())?;
			return Err(());
		};

		let y = split.next();
		let y = y.and_then(|x| u32::from_str_radix(x, 10).ok());

		c.word_len = Some((x, y));

		Ok(())
	}),
	("sentence", Some('s'), &|c, a, e| {
		let Ok(arg) = a() else {
			write!(e, "sentence: missing parameter").map_err(|_| ())?;
			return Err(());
		};

		c.which = ConfigWhich::Sentence;

		let mut split = arg.split(":");

		let Some(x) = split.next() else {
			unreachable!();
		};

		if x == "_" {
			c.sentence_len = None;
			return Ok(());
		}

		let Ok(x) = u32::from_str_radix(x, 10) else {
			write!(e, "sentences: unparsable input").map_err(|_| ())?;
			return Err(());
		};

		let y = split.next();
		let y = y.and_then(|x| u32::from_str_radix(x, 10).ok());

		c.sentence_len = Some((x, y));

		Ok(())
	}),
	("paragraph", Some('p'), &|c, a, e| {
		let Ok(arg) = a() else {
			write!(e, "paragraph: missing parameter").map_err(|_| ())?;
			return Err(());
		};

		c.which = ConfigWhich::Paragraph;

		let mut split = arg.split(":");

		let Some(x) = split.next() else {
			unreachable!();
		};

		if x == "_" {
			c.paragraph_len = None;
			return Ok(());
		}

		let Ok(x) = u32::from_str_radix(x, 10) else {
			write!(e, "paragraph: unparsable input").map_err(|_| ())?;
			return Err(());
		};

		let y = split.next();
		let y = y.and_then(|x| u32::from_str_radix(x, 10).ok());

		c.paragraph_len = Some((x, y));

		Ok(())
	}),
];

const HELP: &str = "\
Usage: salps [OPTION]...
Generate text.
  -w, --word [x(:y?)]
                    if y is set, set word length between x and y
					otherwise, set word length to x. if x is '_', random (default)
  -s, --sentence [x(:y?)]
                    if y is set, set sentence length between x and y
					otherwise, set sentence length to x. if x is '_', random (default)
  -p, --paragraph [x(:y?)]
                    if y is set, set paragraph length between x and y
					otherwise, set paragraph length to x. if x is '_', random (default)

                    the last option will be the option generated.
                    default: --paragraph _

  -d, --seed [x]    seed. if x is '_', random (default)
                    
      --help        display this help and exit
      --version     display version information and exit
";

const VERSION: &str = "\
salps (goreutils) 0.1
Copyright (C) 2025 Everyone, except Author.
License GLWT
Everyone is permitted to copy, distribute, modify, merge, sell, publish,
sublicense or whatever they want with this software but at their OWN RISK
<https://github.com/me-shaon/GLWTPL/blob/master/LICENSE>
";

fn main() {
	let out = args::quick(RULES);

	let (config, _) = match out {
		Ok(x) => x,
		Err(e) => {
			eprintln!("salps: {}", e);
			eprintln!("Try 'salps --help' for more information.");
			return;
		},
	};

	if config.help {
		print!("{}", HELP);
		return;
	}

	if config.version {
		print!("{}", VERSION);
		return;
	}

	struct Wrap(lykoi_data::rng::XorShift64);
	impl salps::Rand for Wrap {
		fn next(&mut self) -> f64 {
			self.0.nextf()
		}
	}

	let seed = config.seed.unwrap_or_else(|| getrandom::u64().unwrap_or_else(|_| goreutils::util::gen_time()));
	let mut rng = Wrap(lykoi_data::rng::XorShift64::new(seed));

	let salps_config = salps::Config {
		word_len: config.word_len,
		sentence_len: config.sentence_len,
		paragraph_len: config.paragraph_len,
	};

	let out = match config.which {
		ConfigWhich::Word => salps::word(&mut rng, &salps_config),
		ConfigWhich::Sentence => salps::sentence(&mut rng, &salps_config),
		ConfigWhich::Paragraph => salps::paragraph(&mut rng, &salps_config),
	};

	println!("{}", out);
}

