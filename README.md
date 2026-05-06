# dumpx

Generate sample files in multiple formats and target sizes. `dumpx` is a Rust
binary with human-readable and agent-friendly output.

> Beta release: use with caution.

## Installation

Install from this local checkout:

```sh
cargo install --path .
```

Cargo installs the binary to `~/.cargo/bin/dumpx`. Make sure that directory is
on your `PATH`:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
```

For zsh, add that line to `~/.zshrc`.

You can also build and place the binary yourself:

```sh
cargo build --release
cp target/release/dumpx /usr/local/bin/
```

Uninstall a Cargo-installed binary:

```sh
cargo uninstall dumpx
```

## Usage

After installation, run `dumpx` directly:

Run without arguments to use the interactive prompt:

```sh
dumpx
```

```sh
dumpx --size 10KiB,100KiB,1MiB
```

Short positional form:

```sh
dumpx csv 10MB
dumpx csv 100MB
dumpx csv,json 100MB
dumpx csv,json 100MB,2MB
dumpx csv 10MB .
dumpx 100MB csv
```

Generate only selected formats:

```sh
dumpx --format csv,parquet,jsonl,png --size 1MiB,10MiB
```

Attach tags to filenames, file contents, and stdout reports:

```sh
dumpx --size 100KiB --format csv,jsonl --tag suite=smoke --tag owner=agent
```

Agent-friendly output:

```sh
dumpx --json -s 100KiB -f txt -t run=ci
```

Machine-readable summary JSON:

```sh
dumpx --output json --size 1MiB --format parquet,png
```

`--json` is shorthand for the more explicit `--quiet --output json`.

Template-driven fake data is supported for `csv`, `json`, `jsonl`, `txt`, `md`,
and `pdf`:

```sh
dumpx csv 1MiB --template-header "id,name,email" --template "{{id}},{{name}},{{email}}"
dumpx jsonl 1MiB --template '{"id":{{id}},"name":"{{name}}","email":"{{email}}"}'
dumpx txt 1MiB --template-file row-template.txt
```

Supported template placeholders use `{{name}}` syntax. Common placeholders:

```text
id, row, bool, number, digit, number_format, ssn
uuid, uuid_v1, uuid_v3, uuid_v4, uuid_v5, uuid_v6, uuid_v7, uuid_v8
name, name_with_title, first_name, last_name, title, prefix, suffix
email, safe_email, free_email, email_provider, domain_suffix, username, password
ipv4, ipv6, ip, mac, mac_address, user_agent
company, company_name, company_suffix, buzzword, catch_phrase, bs, profession, industry
job, job_title, seniority, field, job_field, position, job_position
phone, phone_number, cell, cell_phone, cell_number
city, country, country_code, street, street_name, state, state_abbr, zip, postcode
building_number, latitude, longitude, geohash, timezone
word, words, sentence, sentences, paragraph, paragraphs
markdown_italic, markdown_bold, markdown_link, markdown_bullets, markdown_items
markdown_quote, markdown_multiline_quote, markdown_code
isbn, isbn10, isbn13, credit_card, currency_code, currency_name, currency_symbol, bic, isin
file_path, file_name, file_extension, dir_path, mime_type, semver
image_url, image_seed_url, image_grayscale_url, image_blur_url
hex_color, rgb_color, rgba_color, hsl_color, hsla_color, color
http_status, valid_http_status, date, time, datetime, time_date, time_time, time_datetime
```

Build the binary:

```sh
cargo build --release
./target/release/dumpx --help
```

During local development, prefix examples with `cargo run --`, for example:

```sh
cargo run -- csv 10MB
```

Safety defaults:

```sh
# Existing files are refused by default.
dumpx txt 1MiB
dumpx txt 1MiB --force

# Each file is capped at 1GiB unless explicitly allowed.
dumpx txt 2GiB --allow-large
```

Supported formats:

```text
csv, parquet, json, jsonl, txt, md, pdf, jpg, png
```

Size units are case-sensitive for bytes versus bits: `B` means bytes and `b`
means bits. All K/M/G prefixes use powers of 1024. For bytes, `kB`, `KB`, and
`KiB` are equivalent; `MB` and `MiB` are equivalent; `GB` and `GiB` are
equivalent. For bits, `kb`, `Kb`, and `Kib` are equivalent; `Mb` and `Mib` are
equivalent; `Gb` and `Gib` are equivalent.

Generated files meet or exceed the requested target size. Compressed and
container formats such as Parquet, JPG, and PNG may overshoot because their exact
encoded size depends on format overhead and compression behavior.

## License

MIT. See [LICENSE](LICENSE).
