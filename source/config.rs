//! Generate fonts.conf (local) and 42-repl-global.conf (global) — byte-for-byte ports of the Python templates.

use std::fs;
use std::io::Write;
use std::path::Path;

use crate::lprintln;

pub fn write_fonts_conf(
    csgo_fonts: &Path,
    safe_font_name: &str,
    ui_scale: f64,
    success_msg: &str,
    error_msg: &str,
) {
    let content = format!(
        r#"<?xml version='1.0'?>
<!DOCTYPE fontconfig SYSTEM 'fonts.dtd'>
<fontconfig>

	<dir prefix="default">../../csgo/panorama/fonts</dir>
	<dir>WINDOWSFONTDIR</dir>
	<dir>~/.fonts</dir>
	<dir>/usr/share/fonts</dir>
	<dir>/usr/local/share/fonts</dir>
	<dir prefix="xdg">fonts</dir>

	<fontpattern>Arial</fontpattern>
	<fontpattern>.uifont</fontpattern>
	<fontpattern>notosans</fontpattern>
	<fontpattern>notoserif</fontpattern>
	<fontpattern>notomono-regular</fontpattern>
	<fontpattern>{safe_font_name}</fontpattern>
	<fontpattern>.ttf</fontpattern>
	<fontpattern>FONTFILENAME</fontpattern>
	
	<cachedir>WINDOWSTEMPDIR_FONTCONFIG_CACHE</cachedir>
	<cachedir>~/.fontconfig</cachedir>

	<!-- Vietnamese language support -->
	<match>
		<test name="lang">
			<string>vi-vn</string>
		</test>
		<test name="family" compare="contains">
			<string>Stratum2</string>
		</test>
		<test qual="all" name="family" compare="not_contains">
			<string>TF</string>
		</test>
		<test qual="all" name="family" compare="not_contains">
			<string>Mono</string>
		</test>
		<test qual="all" name="family" compare="not_contains">
			<string>ForceStratum2</string>
		</test>
		<edit name="weight" mode="assign">
			<if>
				<contains>
					<name>family</name>
					<string>Stratum2 Black</string>
				</contains>
				<int>210</int>
				<name>weight</name>
			</if>
		</edit>
		<edit name="slant" mode="assign">
			<if>
				<contains>
					<name>family</name>
					<string>Italic</string>
				</contains>
				<int>100</int>
				<name>slant</name>
			</if>
		</edit>
		<edit name="pixelsize" mode="assign">
			<if>
				<or>
					<contains>
						<name>family</name>
						<string>Condensed</string>
					</contains>
					<less_eq>
						<name>width</name>
						<int>75</int>
					</less_eq>
				</or>
				<times>
					<name>pixelsize</name>
					<double>0.7</double>
				</times>
				<times>
					<name>pixelsize</name>
					<double>0.9</double>
				</times>
			</if>
		</edit>
		<edit name="family" mode="assign" binding="same">
			<string>notosans</string>
		</edit>
	</match>

	<selectfont> 
		<rejectfont> 
			<pattern> 
				<patelt name="fontformat" > 
					<string>Type 1</string> 
				</patelt> 
			</pattern> 
		</rejectfont> 
	</selectfont> 

	<match target="font" >
		<edit name="embeddedbitmap" mode="assign">
			<bool>false</bool>
		</edit>
	</match>

	<match target="pattern" >
		<edit name="prefer_outline" mode="assign">
			<bool>true</bool>
		</edit>
	</match>

	<match target="pattern" >
		<edit name="do_substitutions" mode="assign">
			<bool>true</bool>
		</edit>
	</match>

	<match target="font">
		<edit name="force_autohint" mode="assign">
			<bool>false</bool>
		</edit>
	</match>

	<include>../../../core/panorama/fonts/conf.d</include>

	<!-- Adjust HUD font size (Money, Health, Ammo) -->
	<match target="font">
		<test name="family" compare="contains">
			<string>Stratum2</string>
		</test>
		<test name="family" compare="contains">
			<string>{safe_font_name}</string>
		</test>
		<edit name="pixelsize" mode="assign">
			<times>
				<name>pixelsize</name>
				<double>{ui_scale}</double>
			</times>
		</edit>
	</match>
	
	<!-- Custom fonts -->
	<match>
		<test name="family">
			<string>Stratum2</string>
		</test>
		<edit name="family" mode="append" binding="strong">
			<string>{safe_font_name}</string>
		</edit>
	</match>
	
	<match>
		<test name="family">
			<string>Stratum2 Bold</string>
		</test>
		<edit name="family" mode="append" binding="strong">
			<string>{safe_font_name}</string>
		</edit>
	</match>
	
	<match>
		<test name="family">
			<string>Arial</string>
		</test>
		<edit name="family" mode="append" binding="strong">
			<string>{safe_font_name}</string>
		</edit>
	</match>
	
	<match>
		<test name="family">
			<string>Times New Roman</string>
		</test>
		<edit name="family" mode="append" binding="strong">
			<string>{safe_font_name}</string>
		</edit>
	</match>
	
	<match>
		<test name="family">
			<string>Courier New</string>
		</test>
		<edit name="family" mode="append" binding="strong">
			<string>{safe_font_name}</string>
		</edit>
	</match>

	<match>
		<test name="family">
			<string>notosans</string>
		</test>
		<edit name="family" mode="append" binding="strong">
			<string>{safe_font_name}</string>
		</edit>
	</match>
	
	<match>
		<test name="family">
			<string>notoserif</string>
		</test>
		<edit name="family" mode="append" binding="strong">
			<string>{safe_font_name}</string>
		</edit>
	</match>
	
	<match>
		<test name="family">
			<string>notomono-regular</string>
		</test>
		<edit name="family" mode="append" binding="strong">
			<string>{safe_font_name}</string>
		</edit>
	</match>

	<!-- Custom fonts: catch-all for any Stratum2 variant (clan tag, nav bar, etc.) -->
	<match target="pattern">
		<test name="family" compare="contains">
			<string>Stratum2</string>
		</test>
		<edit name="family" mode="assign" binding="strong">
			<string>{safe_font_name}</string>
		</edit>
	</match>

	<!-- Custom fonts: unconditional fallback, replace everything else -->
	<match target="pattern">
		<edit name="family" mode="assign" binding="strong">
			<string>{safe_font_name}</string>
		</edit>
	</match>

</fontconfig>"#
    );

    let path = csgo_fonts.join("fonts.conf");
    match fs::File::create(&path).and_then(|mut f| f.write_all(content.as_bytes())) {
        Ok(_) => lprintln!("{}", success_msg),
        Err(e) => crate::print_error(error_msg, Some(&e)),
    }
}

pub fn write_repl_conf(
    core_fonts: &Path,
    safe_font_name: &str,
    ui_scale: f64,
    success_msg: &str,
    error_msg: &str,
) {
    let mut content = format!(
        r#"<?xml version='1.0'?>
<!DOCTYPE fontconfig SYSTEM 'fonts.dtd'>
<fontconfig>

	<match target="font">
		<test name="family" compare="contains"><string>Stratum2</string></test>
		<edit name="pixelsize" mode="assign">
			<times><name>pixelsize</name><double>{ui_scale}</double></times>
		</edit>
	</match>
	<match target="font">
		<test name="family" compare="contains"><string>{safe_font_name}</string></test>
		<edit name="pixelsize" mode="assign">
			<times><name>pixelsize</name><double>{ui_scale}</double></times>
		</edit>
	</match>
"#
    );

    let fonts_to_replace = [
        "Stratum2",
        "Stratum2 Bold",
        "Arial",
        "Times New Roman",
        "Courier New",
        "notosans",
        "notoserif",
        "notomono-regular",
        "noto",
    ];
    for font_to_repl in fonts_to_replace {
        content.push_str(&format!(
            r#"
	<match target="font">
		<test name="family"><string>{font_to_repl}</string></test>
		<edit name="family" mode="assign"><string>{safe_font_name}</string></edit>
	</match>
	<match target="pattern">
		<test name="family"><string>{font_to_repl}</string></test>
		<edit name="family" mode="prepend" binding="strong"><string>{safe_font_name}</string></edit>
	</match>"#
        ));
    }

    // Catch-all: replace any Stratum2 variant (clan tag, nav bar, monodigit, etc.)
    content.push_str(&format!(
        r#"
	<match target="font">
		<test name="family" compare="contains"><string>Stratum2</string></test>
		<edit name="family" mode="assign"><string>{safe_font_name}</string></edit>
	</match>
	<match target="pattern">
		<test name="family" compare="contains"><string>Stratum2</string></test>
		<edit name="family" mode="assign" binding="strong"><string>{safe_font_name}</string></edit>
	</match>

	<!-- Unconditional fallback: replace any remaining font request -->
	<match target="pattern">
		<edit name="family" mode="assign" binding="strong"><string>{safe_font_name}</string></edit>
	</match>"#
    ));

    content.push_str("\n</fontconfig>");

    let path = core_fonts.join("42-repl-global.conf");
    match fs::File::create(&path).and_then(|mut f| f.write_all(content.as_bytes())) {
        Ok(_) => lprintln!("{}", success_msg),
        Err(e) => crate::print_error(error_msg, Some(&e)),
    }
}
