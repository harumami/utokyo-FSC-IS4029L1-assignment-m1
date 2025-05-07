glob 'examples/*.toml' | each { |path| open --raw $path | cargo run --release -- toml webp | save -f $'examples/($path | path parse | $in.stem).webp' }
