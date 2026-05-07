# dumpx

<img width="2172" height="724" alt="dumpx-banner-min" src="https://github.com/user-attachments/assets/d851d668-1867-4ac4-a004-d77be1fd0995" />


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

Use a custom file name for a single generated file:

```sh
dumpx csv 10MB --name users.csv
```

Use a custom file-name template for multi-format or multi-size runs:

```sh
dumpx csv,json 1MiB,2MiB --name "{format}-{size}.{extension}"
```

`--name` accepts `{prefix}`, `{format}`, `{size}`, `{extension}`, and `{index}`.
It names files inside the output directory only, so path separators such as `/`
and `\` are rejected. If a custom name would produce the same output path for
more than one generated file, the run is rejected; add `{format}`, `{size}`, or
`{index}` to make names unique.

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
Summary reports include `planned_count` for the number of files requested and
`count` for the number generated.

Generate multiple files for each requested format and size pair:

```sh
dumpx csv 10MB --number-of-files=3
dumpx csv,json 1MiB,2MiB --number-of-files=2
```

Template-driven fake data is supported for `csv`, `json`, `jsonl`, `txt`, `md`,
and `pdf`:

```sh
dumpx csv 1MiB --template-header "id,name,email" --template "{{id}},{{name}},{{email}}"
dumpx jsonl 1MiB --template '{"id":{{id}},"name":"{{name}}","email":"{{email}}"}'
dumpx txt 1MiB --template-file row-template.txt
```

Supported template placeholders use `{{name}}` syntax.

Template placeholder reference:

```text
Core:
id, row, bool, boolean, number, digit, number_format, ssn

UUID:
uuid, uuid_v1, uuid_v3, uuid_v4, uuid_v5, uuid_v6, uuid_v7, uuid_v8

Name:
name, name_with_title, first_name, last_name, title, name_title, prefix, suffix,
name_suffix

Internet:
email, safe_email, free_email, email_provider, free_email_provider,
domain_suffix, username, password, ipv4, ipv6, ip, mac, mac_address, user_agent

Company:
company, company_name, company_suffix, buzzword, buzzword_middle, buzzword_tail,
catch_phrase, bs, bs_verb, bs_adj, bs_noun, profession, industry

Job:
job, job_title, seniority, field, job_field, position, job_position

Phone:
phone, phone_number, cell, cell_phone, cell_number

Address:
city_prefix, city_suffix, city, country, country_code, street_suffix, street,
street_name, timezone, time_zone, state, state_name, state_abbr,
secondary_address_type, secondary_address, zip, zip_code, postcode, post_code,
building_number, latitude, longitude, geohash

Lorem:
word, words, sentence, sentences, paragraph, paragraphs

Markdown:
markdown_italic, italic_word, markdown_bold, bold_word, markdown_link,
markdown_bullets, bullet_points, markdown_items, list_items, markdown_quote,
blockquote, markdown_multiline_quote, markdown_code, code

Barcode, finance, and currency:
isbn, isbn10, isbn_10, isbn13, isbn_13, credit_card, credit_card_number,
currency_code, currency_name, currency_symbol, bic, isin

Filesystem:
file_path, file_name, file_extension, dir_path, mime_type, semver,
semver_stable, semver_unstable

Image URLs:
image_url, picsum, image_seed_url, picsum_seed, image_grayscale_url,
picsum_grayscale, image_blur_url, picsum_blur

Color:
hex_color, rgb_color, rgba_color, hsl_color, hsla_color, color

HTTP:
http_status, rfc_http_status, valid_http_status

Date and time:
date, chrono_date, time, chrono_time, datetime, chrono_datetime, time_date,
time_time, time_datetime
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
