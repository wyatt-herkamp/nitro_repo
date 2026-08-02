//! # nr-badge
//!
//! SVG badge rendering for services like [shields.io](https://shields.io/), verified to match the
//! npm [badge-maker](https://www.npmjs.com/package/badge-maker) 1-to-1 by side-by-side rendering
//! tests*.
//!
//! ## Provenance
//!
//! Vendored from <https://github.com/wyatt-herkamp/badge-maker> (branch
//! `updates_and_improvements`, rev `e0dc481`), itself a fork of the unmaintained
//! <https://github.com/cgbur/badge-maker> by Chris Burgess. MIT licensed — see `LICENSE`.
//!
//! Changes made while vendoring: the `badge-maker` CLI binary and its `clap` dependency are gone,
//! `once_cell::sync::Lazy` became [`std::sync::LazyLock`], and only the Verdana 11px font width
//! table is carried (the other three were already unused). Text measurement is unchanged, so
//! rendered output is byte-identical to the fork.
//!
//! <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="84" height="20" role="img" aria-label="example: flat"><title>example: flat</title><linearGradient id="bms-6df9790b166df7b8" x2="0" y2="100%"><stop offset="0" stop-color="#bbb" stop-opacity=".1"/><stop offset="1" stop-opacity=".1"/></linearGradient><clipPath id="bmr-6df9790b166df7b8"><rect width="84" height="20" rx="3" fill="#fff"/></clipPath><g clip-path="url(#bmr-6df9790b166df7b8)"><rect width="57" height="20" fill="#555"/><rect x="57" width="27" height="20" fill="#ff5b5a"/><rect width="84" height="20" fill="url(#bms-6df9790b166df7b8)"/></g><g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" text-rendering="geometricPrecision" font-size="110"><text aria-hidden="true" x="295" y="150" fill="#010101" fill-opacity=".3" transform="scale(.1)" textLength="470">example</text><text x="295" y="140" transform="scale(.1)" fill="#fff" textLength="470">example</text><text aria-hidden="true" x="695" y="150" fill="#010101" fill-opacity=".3" transform="scale(.1)" textLength="170">flat</text><text x="695" y="140" transform="scale(.1)" fill="#fff" textLength="170">flat</text></g></svg>
//!
//! <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="102" height="18" role="img" aria-label="example: plastic"><title>example: plastic</title><linearGradient id="bms-673fc3b0d46c7e6f" x2="0" y2="100%"><stop offset="0"  stop-color="#fff" stop-opacity=".7"/><stop offset=".1" stop-color="#aaa" stop-opacity=".1"/><stop offset=".9" stop-color="#000" stop-opacity=".3"/><stop offset="1"  stop-color="#000" stop-opacity=".5"/></linearGradient><clipPath id="bmr-673fc3b0d46c7e6f"><rect width="102" height="18" rx="4" fill="#fff"/></clipPath><g clip-path="url(#bmr-673fc3b0d46c7e6f)"><rect width="57" height="18" fill="#555"/><rect x="57" width="45" height="18" fill="#ffb932"/><rect width="102" height="18" fill="url(#bms-673fc3b0d46c7e6f)"/></g><g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" text-rendering="geometricPrecision" font-size="110"><text aria-hidden="true" x="295" y="140" fill="#010101" fill-opacity=".3" transform="scale(.1)" textLength="470">example</text><text x="295" y="130" transform="scale(.1)" fill="#fff" textLength="470">example</text><text aria-hidden="true" x="785" y="140" fill="#ccc" fill-opacity=".3" transform="scale(.1)" textLength="350">plastic</text><text x="785" y="130" transform="scale(.1)" fill="#333" textLength="350">plastic</text></g></svg>
//!
//! <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="122" height="20" role="img" aria-label="example: flatsquare"><title>example: flatsquare</title><g shape-rendering="crispEdges"><rect width="57" height="20" fill="#555"/><rect x="57" width="65" height="20" fill="#fffe27"/></g><g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" text-rendering="geometricPrecision" font-size="110"><text x="295" y="140" transform="scale(.1)" fill="#fff" textLength="470">example</text><text x="885" y="140" transform="scale(.1)" fill="#333" textLength="550">flatsquare</text></g></svg>
//!
//! <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="88" height="20" role="img" aria-label="badge: maker"><title>badge: maker</title><linearGradient id="bms-65d9c12cf1a1b6af" x2="0" y2="100%"><stop offset="0" stop-color="#bbb" stop-opacity=".1"/><stop offset="1" stop-opacity=".1"/></linearGradient><clipPath id="bmr-65d9c12cf1a1b6af"><rect width="88" height="20" rx="3" fill="#fff"/></clipPath><g clip-path="url(#bmr-65d9c12cf1a1b6af)"><rect width="43" height="20" fill="#555"/><rect x="43" width="45" height="20" fill="#33b5e5"/><rect width="88" height="20" fill="url(#bms-65d9c12cf1a1b6af)"/></g><g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" text-rendering="geometricPrecision" font-size="110"><text aria-hidden="true" x="225" y="150" fill="#010101" fill-opacity=".3" transform="scale(.1)" textLength="330">badge</text><text x="225" y="140" transform="scale(.1)" fill="#fff" textLength="330">badge</text><text aria-hidden="true" x="645" y="150" fill="#010101" fill-opacity=".3" transform="scale(.1)" textLength="350">maker</text><text x="645" y="140" transform="scale(.1)" fill="#fff" textLength="350">maker</text></g></svg>
//!
//! <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="94" height="20" role="img" aria-label="color: example"><title>color: example</title><linearGradient id="bms-e0307dea9033836d" x2="0" y2="100%"><stop offset="0" stop-color="#bbb" stop-opacity=".1"/><stop offset="1" stop-opacity=".1"/></linearGradient><clipPath id="bmr-e0307dea9033836d"><rect width="94" height="20" rx="3" fill="#fff"/></clipPath><g clip-path="url(#bmr-e0307dea9033836d)"><rect width="37" height="20" fill="#555"/><rect x="37" width="57" height="20" fill="#0ac832"/><rect width="94" height="20" fill="url(#bms-e0307dea9033836d)"/></g><g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" text-rendering="geometricPrecision" font-size="110"><text aria-hidden="true" x="195" y="150" fill="#010101" fill-opacity=".3" transform="scale(.1)" textLength="270">color</text><text x="195" y="140" transform="scale(.1)" fill="#fff" textLength="270">color</text><text aria-hidden="true" x="645" y="150" fill="#010101" fill-opacity=".3" transform="scale(.1)" textLength="470">example</text><text x="645" y="140" transform="scale(.1)" fill="#fff" textLength="470">example</text></g></svg>
//!
//! <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="100" height="20" role="img" aria-label="example: badge"><title>example: badge</title><linearGradient id="bms-058ad8d642fc4c85" x2="0" y2="100%"><stop offset="0" stop-color="#bbb" stop-opacity=".1"/><stop offset="1" stop-opacity=".1"/></linearGradient><clipPath id="bmr-058ad8d642fc4c85"><rect width="100" height="20" rx="3" fill="#fff"/></clipPath><g clip-path="url(#bmr-058ad8d642fc4c85)"><rect width="57" height="20" fill="#555"/><rect x="57" width="43" height="20" fill="#4c1"/><rect width="100" height="20" fill="url(#bms-058ad8d642fc4c85)"/></g><g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" text-rendering="geometricPrecision" font-size="110"><text aria-hidden="true" x="295" y="150" fill="#010101" fill-opacity=".3" transform="scale(.1)" textLength="470">example</text><text x="295" y="140" transform="scale(.1)" fill="#fff" textLength="470">example</text><text aria-hidden="true" x="775" y="150" fill="#010101" fill-opacity=".3" transform="scale(.1)" textLength="330">badge</text><text x="775" y="140" transform="scale(.1)" fill="#fff" textLength="330">badge</text></g></svg>
//!
//! *_This library differs in that it generates unique IDs for the svg so it can be directly
//!  embedded in websites (such as in this doc_svgs). So a diff between the outputs will not match. We
//! only claim the visual outputs match which is whats important._
//!
//! ### Example
//! <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="88" height="20" role="img" aria-label="badge: maker"><title>badge: maker</title><linearGradient id="bms-badgemaker55533b5e5" x2="0" y2="100%"><stop offset="0" stop-color="#bbb" stop-opacity=".1"/><stop offset="1" stop-opacity=".1"/></linearGradient><clipPath id="bmr-badgemaker55533b5e5"><rect width="88" height="20" rx="3" fill="#fff"/></clipPath><g clip-path="url(#bmr-badgemaker55533b5e5)"><rect width="43" height="20" fill="#555"/><rect x="43" width="45" height="20" fill="#33b5e5"/><rect width="88" height="20" fill="url(#bms-badgemaker55533b5e5)"/></g><g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" text-rendering="geometricPrecision" font-size="110"><text aria-hidden="true" x="225" y="150" fill="#010101" fill-opacity=".3" transform="scale(.1)" textLength="330">badge</text><text x="225" y="140" transform="scale(.1)" fill="#fff" textLength="330">badge</text><text aria-hidden="true" x="645" y="150" fill="#010101" fill-opacity=".3" transform="scale(.1)" textLength="350">maker</text><text x="645" y="140" transform="scale(.1)" fill="#fff" textLength="350">maker</text></g></svg>
//! ```rust
//! use nr_badge::BadgeBuilder;
//!
//! let svg = BadgeBuilder::new()
//!       .label("badge")
//!       .message("maker")
//!       .color_parse("#33B5E5")
//!       .build()?
//!       .svg();
//!
//! println!("{}", svg);
//!
//! # Ok::<(), nr_badge::error::Error>(())
//! ```
//!
//! ## Features
//!
//! Different [styles](Style), [colors](color::Color), [logos](Logo), and [links](Links). The
//! [badge builder](BadgeBuilder) accepts all of these options with the `field()` and an
//! alternate method of `field_parse()` which accepts a string and will attempt parse the text as
//! a valid field.
//!
//! ### [Colors](Color)
//!
//! We currently support hex colors 3 and 6 chars long, [named colors](color::NamedColor)
//! and their [alias's](color::AliasColor), and [RGB](color::Color::Rgb) color inputs. These can be constructed
//! with their enum variants or using the `...parse()` methods.
//!
//! <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="94" height="20" role="img" aria-label="color: example"><title>color: example</title><linearGradient id="bms-e0307dea9033836d" x2="0" y2="100%"><stop offset="0" stop-color="#bbb" stop-opacity=".1"/><stop offset="1" stop-opacity=".1"/></linearGradient><clipPath id="bmr-e0307dea9033836d"><rect width="94" height="20" rx="3" fill="#fff"/></clipPath><g clip-path="url(#bmr-e0307dea9033836d)"><rect width="37" height="20" fill="#555"/><rect x="37" width="57" height="20" fill="#0ac832"/><rect width="94" height="20" fill="url(#bms-e0307dea9033836d)"/></g><g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" text-rendering="geometricPrecision" font-size="110"><text aria-hidden="true" x="195" y="150" fill="#010101" fill-opacity=".3" transform="scale(.1)" textLength="270">color</text><text x="195" y="140" transform="scale(.1)" fill="#fff" textLength="270">color</text><text aria-hidden="true" x="645" y="150" fill="#010101" fill-opacity=".3" transform="scale(.1)" textLength="470">example</text><text x="645" y="140" transform="scale(.1)" fill="#fff" textLength="470">example</text></g></svg>
//! ```rust
//! use nr_badge::BadgeBuilder;
//! use nr_badge::color::{Color, AliasColor, NamedColor};
//!
//! let svg = BadgeBuilder::new()
//!     .label("color")
//!     .message("example")
//!     // by enums
//!     .color(Color::Named(NamedColor::BrightGreen))
//!     .color(Color::Alias(AliasColor::Success))
//!     .color(Color::Rgb(10, 200, 50))
//!     // or parsing
//!     .color_parse("brightgreen")
//!     .color_parse("success")
//!     .color_parse("rgb(10, 200, 50)")
//!     .build()?
//!     .svg();
//!
//! # Ok::<(), nr_badge::error::Error>(())
//! ```
//!
//!
//! ### [Styles](Style)
//! **Supported**. Others coming soon. See [Style](Style) enum for choices when
//! building or use the string literals.
//!
//!  - **Flat** <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="84" height="20" role="img" aria-label="example: flat"><title>example: flat</title><linearGradient id="bms-exampleflat555ff5b5a" x2="0" y2="100%"><stop offset="0" stop-color="#bbb" stop-opacity=".1"/><stop offset="1" stop-opacity=".1"/></linearGradient><clipPath id="bmr-exampleflat555ff5b5a"><rect width="84" height="20" rx="3" fill="#fff"/></clipPath><g clip-path="url(#bmr-exampleflat555ff5b5a)"><rect width="57" height="20" fill="#555"/><rect x="57" width="27" height="20" fill="#ff5b5a"/><rect width="84" height="20" fill="url(#bms-exampleflat555ff5b5a)"/></g><g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" text-rendering="geometricPrecision" font-size="110"><text aria-hidden="true" x="295" y="150" fill="#010101" fill-opacity=".3" transform="scale(.1)" textLength="470">example</text><text x="295" y="140" transform="scale(.1)" fill="#fff" textLength="470">example</text><text aria-hidden="true" x="695" y="150" fill="#010101" fill-opacity=".3" transform="scale(.1)" textLength="170">flat</text><text x="695" y="140" transform="scale(.1)" fill="#fff" textLength="170">flat</text></g></svg>
//!  - **Plastic** <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="102" height="18" role="img" aria-label="example: plastic"><title>example: plastic</title><linearGradient id="bms-673fc3b0d46c7e6f" x2="0" y2="100%"><stop offset="0"  stop-color="#fff" stop-opacity=".7"/><stop offset=".1" stop-color="#aaa" stop-opacity=".1"/><stop offset=".9" stop-color="#000" stop-opacity=".3"/><stop offset="1"  stop-color="#000" stop-opacity=".5"/></linearGradient><clipPath id="bmr-673fc3b0d46c7e6f"><rect width="102" height="18" rx="4" fill="#fff"/></clipPath><g clip-path="url(#bmr-673fc3b0d46c7e6f)"><rect width="57" height="18" fill="#555"/><rect x="57" width="45" height="18" fill="#ffb932"/><rect width="102" height="18" fill="url(#bms-673fc3b0d46c7e6f)"/></g><g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" text-rendering="geometricPrecision" font-size="110"><text aria-hidden="true" x="295" y="140" fill="#010101" fill-opacity=".3" transform="scale(.1)" textLength="470">example</text><text x="295" y="130" transform="scale(.1)" fill="#fff" textLength="470">example</text><text aria-hidden="true" x="785" y="140" fill="#ccc" fill-opacity=".3" transform="scale(.1)" textLength="350">plastic</text><text x="785" y="130" transform="scale(.1)" fill="#333" textLength="350">plastic</text></g></svg>
//!  - **FlatSquare** <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="122" height="20" role="img" aria-label="example: flatsquare"><title>example: flatsquare</title><g shape-rendering="crispEdges"><rect width="57" height="20" fill="#555"/><rect x="57" width="65" height="20" fill="#fffe27"/></g><g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" text-rendering="geometricPrecision" font-size="110"><text x="295" y="140" transform="scale(.1)" fill="#fff" textLength="470">example</text><text x="885" y="140" transform="scale(.1)" fill="#333" textLength="550">flatsquare</text></g></svg>
//!  - ForTheBadge
//!  - Social
//!
//! ```rust
//! use nr_badge::{BadgeBuilder, Style};
//!
//! let svg = BadgeBuilder::new()
//!   .label("example")
//!   .message("plastic")
//!   .color_parse("#FFB932")
//!   .style(Style::Plastic) // example of using typed input
//!   .style_parse("plastic") // example of parsing to derive
//!   .build()?
//!   .svg();
//!
//! println!("{}", svg);
//!
//! # Ok::<(), nr_badge::error::Error>(())
//! ```
//!
//! ### [Links](Links) & [Logos](Logo)
//! Adding links to the natively rendered badge supported. This is great if you need
//! to embed the svg directly. However, on a website like the rust docs they may show
//! the underline. To solve this, your third-party api that renders the badges should
//! wrap the svg in markdown `[![name for readers](link to api endpoint)](link when clicked)`.
//!
//!
//! <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="81" height="20" ><a target="_blank" xlink:href="https://www.rust-lang.org/"><g shape-rendering="crispEdges"><rect width="50" height="20" fill="#555"/><rect x="50" width="31" height="20" fill="#f5f5f5"/></g><g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" text-rendering="geometricPrecision" font-size="110"><image x="5" y="3" width="14" height="14" xlink:href="https://upload.wikimedia.org/wikipedia/commons/thumb/d/d5/Rust_programming_language_black_logo.svg/1024px-Rust_programming_language_black_logo.svg.png"/><text x="345" y="140" transform="scale(.1)" fill="#fff" textLength="230">lang</text><text x="645" y="140" transform="scale(.1)" fill="#333" textLength="210">rust</text></g></a></svg>
//! ```rust
//! use nr_badge::BadgeBuilder;
//!
//! let logo_url = "https://upload.wikimedia.org/wikipedia/commons/\
//!   thumb/d/d5/Rust_programming_language_black_logo.svg/\
//!   1024px-Rust_programming_language_black_logo.svg.png";
//!
//! let svg = BadgeBuilder::new()
//!   .label("lang")
//!   .message("rust")
//!   .color_parse("#F5F5F5")
//!   .link("https://www.rust-lang.org/")
//!   .logo_url(logo_url)
//!   .style_parse("flatsquare")
//!   .build()?
//!   .svg();
//!
//! # Ok::<(), nr_badge::error::Error>(())
//! ```

pub use badge::{Badge, BadgeBuilder, Links, Logo, Style, color};

pub mod error;

mod badge;
mod render;
