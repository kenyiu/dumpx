//! Simple fake-data template rendering.

use anyhow::{anyhow, Context, Result};
use fake::faker::{
    address::en as address, barcode::en as barcode, boolean::en as boolean,
    chrono::en as chrono_fake, color::en as color, company::en as company,
    creditcard::en as creditcard, currency::en as currency, filesystem::en as filesystem,
    finance::en as finance, http::en as http, internet::en as internet, job::en as job,
    lorem::en as lorem, markdown::en as markdown, name::en as name, number::en as number,
    phone_number::en as phone_number, picsum::en as picsum, time::en as time_fake,
};
use fake::uuid::{UUIDv1, UUIDv3, UUIDv4, UUIDv5, UUIDv6, UUIDv7, UUIDv8};
use fake::Fake;

#[derive(Clone, Debug)]
pub struct Template {
    source: String,
}

#[derive(Clone, Debug, Default)]
pub struct TemplateOptions {
    pub body: Option<Template>,
    pub header: Option<String>,
}

impl Template {
    pub fn new(source: impl Into<String>) -> Result<Self> {
        let source = source.into();
        validate_template(&source)?;
        Ok(Self { source })
    }

    pub fn render(&self, row: u64) -> Result<String> {
        render_template(&self.source, row)
    }
}

pub fn render_template(source: &str, row: u64) -> Result<String> {
    let mut rendered = String::with_capacity(source.len() + 32);
    let mut rest = source;

    while let Some(start) = rest.find("{{") {
        rendered.push_str(&rest[..start]);
        let after_start = &rest[start + 2..];
        let Some(end) = after_start.find("}}") else {
            return Err(anyhow!("template has an unclosed placeholder"));
        };
        let key = after_start[..end].trim();
        if key.is_empty() {
            return Err(anyhow!("template placeholder cannot be empty"));
        }
        rendered.push_str(&fake_value(key, row)?);
        rest = &after_start[end + 2..];
    }

    if rest.contains("}}") {
        return Err(anyhow!(
            "template has a closing placeholder without an opener"
        ));
    }
    rendered.push_str(rest);
    Ok(rendered)
}

fn validate_template(source: &str) -> Result<()> {
    render_template(source, 1).with_context(|| "invalid template")?;
    Ok(())
}

fn fake_value(key: &str, row: u64) -> Result<String> {
    let value = match key {
        "id" | "row" => row.to_string(),
        "bool" | "boolean" => boolean::Boolean(50).fake::<bool>().to_string(),
        "number" => ((row * 7919) % 100_000).to_string(),
        "digit" => number::Digit().fake::<String>(),
        "number_format" | "ssn" => number::NumberWithFormat("###-##-####").fake::<String>(),

        "name" => name::Name().fake::<String>(),
        "name_with_title" => name::NameWithTitle().fake::<String>(),
        "first_name" => name::FirstName().fake::<String>(),
        "last_name" => name::LastName().fake::<String>(),
        "title" | "name_title" | "prefix" => name::Title().fake::<String>(),
        "suffix" | "name_suffix" => name::Suffix().fake::<String>(),

        "email" | "safe_email" => internet::SafeEmail().fake::<String>(),
        "free_email" => internet::FreeEmail().fake::<String>(),
        "email_provider" | "free_email_provider" => internet::FreeEmailProvider().fake::<String>(),
        "domain_suffix" => internet::DomainSuffix().fake::<String>(),
        "username" => internet::Username().fake::<String>(),
        "password" => internet::Password(10..20).fake::<String>(),
        "ipv4" => internet::IPv4().fake::<String>(),
        "ipv6" => internet::IPv6().fake::<String>(),
        "ip" => internet::IP().fake::<String>(),
        "mac" | "mac_address" => internet::MACAddress().fake::<String>(),
        "user_agent" => internet::UserAgent().fake::<String>(),

        "company" | "company_name" => company::CompanyName().fake::<String>(),
        "company_suffix" => company::CompanySuffix().fake::<String>(),
        "buzzword" => company::Buzzword().fake::<String>(),
        "buzzword_middle" => company::BuzzwordMiddle().fake::<String>(),
        "buzzword_tail" => company::BuzzwordTail().fake::<String>(),
        "catch_phrase" => company::CatchPhrase().fake::<String>(),
        "bs" => company::Bs().fake::<String>(),
        "bs_verb" => company::BsVerb().fake::<String>(),
        "bs_adj" => company::BsAdj().fake::<String>(),
        "bs_noun" => company::BsNoun().fake::<String>(),
        "profession" => company::Profession().fake::<String>(),
        "industry" => company::Industry().fake::<String>(),

        "job" | "job_title" => job::Title().fake::<String>(),
        "seniority" => job::Seniority().fake::<String>(),
        "field" | "job_field" => job::Field().fake::<String>(),
        "position" | "job_position" => job::Position().fake::<String>(),

        "phone" | "phone_number" => phone_number::PhoneNumber().fake::<String>(),
        "cell" | "cell_phone" | "cell_number" => phone_number::CellNumber().fake::<String>(),

        "city_prefix" => address::CityPrefix().fake::<String>(),
        "city_suffix" => address::CitySuffix().fake::<String>(),
        "city" => address::CityName().fake::<String>(),
        "country" => address::CountryName().fake::<String>(),
        "country_code" => address::CountryCode().fake::<String>(),
        "street_suffix" => address::StreetSuffix().fake::<String>(),
        "street" | "street_name" => address::StreetName().fake::<String>(),
        "timezone" | "time_zone" => address::TimeZone().fake::<String>(),
        "state" | "state_name" => address::StateName().fake::<String>(),
        "state_abbr" => address::StateAbbr().fake::<String>(),
        "secondary_address_type" => address::SecondaryAddressType().fake::<String>(),
        "secondary_address" => address::SecondaryAddress().fake::<String>(),
        "zip" | "zip_code" => address::ZipCode().fake::<String>(),
        "postcode" | "post_code" => address::PostCode().fake::<String>(),
        "building_number" => address::BuildingNumber().fake::<String>(),
        "latitude" => address::Latitude().fake::<String>(),
        "longitude" => address::Longitude().fake::<String>(),
        "geohash" => address::Geohash(9).fake::<String>(),

        "word" => lorem::Word().fake::<String>(),
        "words" => join_fake(lorem::Words(3..7)),
        "sentence" => lorem::Sentence(4..10).fake::<String>(),
        "sentences" => join_fake(lorem::Sentences(2..4)),
        "paragraph" => lorem::Paragraph(1..3).fake::<String>(),
        "paragraphs" => join_fake(lorem::Paragraphs(2..4)),

        "markdown_italic" | "italic_word" => markdown::ItalicWord().fake::<String>(),
        "markdown_bold" | "bold_word" => markdown::BoldWord().fake::<String>(),
        "markdown_link" => markdown::Link().fake::<String>(),
        "markdown_bullets" | "bullet_points" => join_fake(markdown::BulletPoints(2..5)),
        "markdown_items" | "list_items" => join_fake(markdown::ListItems(2..5)),
        "markdown_quote" | "blockquote" => markdown::BlockQuoteSingleLine(1..2).fake::<String>(),
        "markdown_multiline_quote" => join_fake(markdown::BlockQuoteMultiLine(2..4)),
        "markdown_code" | "code" => markdown::Code(1..2).fake::<String>(),

        "isbn" => barcode::Isbn().fake::<String>(),
        "isbn10" | "isbn_10" => barcode::Isbn10().fake::<String>(),
        "isbn13" | "isbn_13" => barcode::Isbn13().fake::<String>(),
        "credit_card" | "credit_card_number" => creditcard::CreditCardNumber().fake::<String>(),
        "currency_code" => currency::CurrencyCode().fake::<String>(),
        "currency_name" => currency::CurrencyName().fake::<String>(),
        "currency_symbol" => currency::CurrencySymbol().fake::<String>(),
        "bic" => finance::Bic().fake::<String>(),
        "isin" => finance::Isin().fake::<String>(),

        "file_path" => filesystem::FilePath().fake::<String>(),
        "file_name" => filesystem::FileName().fake::<String>(),
        "file_extension" => filesystem::FileExtension().fake::<String>(),
        "dir_path" => filesystem::DirPath().fake::<String>(),
        "mime_type" => filesystem::MimeType().fake::<String>(),
        "semver" => filesystem::Semver().fake::<String>(),
        "semver_stable" => filesystem::SemverStable().fake::<String>(),
        "semver_unstable" => filesystem::SemverUnstable().fake::<String>(),

        "image_url" | "picsum" => picsum::Image().fake::<String>(),
        "image_seed_url" | "picsum_seed" => picsum::ImageWithSeed().fake::<String>(),
        "image_grayscale_url" | "picsum_grayscale" => picsum::ImageGrayscale().fake::<String>(),
        "image_blur_url" | "picsum_blur" => picsum::ImageBlur().fake::<String>(),

        "hex_color" => color::HexColor().fake::<String>(),
        "rgb_color" => color::RgbColor().fake::<String>(),
        "rgba_color" => color::RgbaColor().fake::<String>(),
        "hsl_color" => color::HslColor().fake::<String>(),
        "hsla_color" => color::HslaColor().fake::<String>(),
        "color" => color::Color().fake::<String>(),

        "http_status" | "rfc_http_status" => http::RfcStatusCode().fake::<String>(),
        "valid_http_status" => http::ValidStatusCode().fake::<String>(),

        "date" | "chrono_date" => chrono_fake::Date().fake::<String>(),
        "time" | "chrono_time" => chrono_fake::Time().fake::<String>(),
        "datetime" | "chrono_datetime" => chrono_fake::DateTime().fake::<String>(),
        "time_date" => time_fake::Date().fake::<String>(),
        "time_time" => time_fake::Time().fake::<String>(),
        "time_datetime" => time_fake::DateTime().fake::<String>(),

        "uuid" | "uuid_v4" => UUIDv4.fake::<String>(),
        "uuid_v1" => UUIDv1.fake::<String>(),
        "uuid_v3" => UUIDv3.fake::<String>(),
        "uuid_v5" => UUIDv5.fake::<String>(),
        "uuid_v6" => UUIDv6.fake::<String>(),
        "uuid_v7" => UUIDv7.fake::<String>(),
        "uuid_v8" => UUIDv8.fake::<String>(),
        _ => return Err(anyhow!("unknown template placeholder `{key}`")),
    };
    Ok(sanitize_value(&value))
}

fn join_fake<F>(faker: F) -> String
where
    F: Fake,
    Vec<String>: fake::Dummy<F>,
{
    faker.fake::<Vec<String>>().join(" ")
}

fn sanitize_value(value: &str) -> String {
    value
        .replace(['\r', '\n', '\t'], " ")
        .replace('"', "'")
        .replace(',', " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_known_placeholders() {
        let template = Template::new("{{id}},{{name}},{{email}}").unwrap();
        let rendered = template.render(7).unwrap();
        assert!(rendered.starts_with("7,"));
        assert_eq!(rendered.split(',').count(), 3);
    }

    #[test]
    fn renders_extended_fake_placeholders() {
        let placeholders = [
            "uuid",
            "uuid_v7",
            "ipv4",
            "hex_color",
            "credit_card",
            "currency_code",
            "file_name",
            "image_url",
            "http_status",
            "datetime",
            "time_datetime",
            "markdown_bullets",
            "isbn13",
            "geohash",
        ];

        for placeholder in placeholders {
            let template = Template::new(format!("{{{{{placeholder}}}}}")).unwrap();
            let rendered = template.render(1).unwrap();
            assert!(!rendered.is_empty(), "{placeholder} rendered empty");
            assert!(
                !rendered.contains("{{") && !rendered.contains("}}"),
                "{placeholder} left template markers behind"
            );
        }
    }

    #[test]
    fn rejects_unknown_placeholders() {
        assert!(Template::new("{{wat}}").is_err());
    }
}
