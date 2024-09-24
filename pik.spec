Name: pik
Version: 0.7.0
Release: 1%{?dist}
License: MIT
Summary: Process Interactive Kill is a tool that helps to find and kill process
Url: https://github.com/jacek-kurlit/%{name}
Source0: %{url}/archive/refs/tags/%{version}.tar.gz
BuildRequires: cargo
BuildRequires: rust
BuildRequires: gcc
	
%description
Process Interactive Kill is a command line tool that helps to find and kill process.
It works like pkill command but search is interactive.
	
%prep
%autosetup -p1
	
%build
cargo build --release

%install
install -Dpm 755 target/release/pik %{buildroot}%{_bindir}/pik

rm -f %{buildroot}%{_prefix}/.crates.toml \
    %{buildroot}%{_prefix}/.crates2.json

%files
%license LICENSE
%doc README.md
%{_bindir}/pik
	
%changelog
* Tue Aug 27 2024 Jacek Kurlit <jacek.kurlit@kitopi.com> 0.6.3-1
- fixed fedora build (jacek.kurlit@kitopi.com)
- fix for fedora build (jacek.kurlit@kitopi.com)

* Tue Aug 27 2024 Jacek Kurlit <jacek.kurlit@kitopi.com> 0.6.1-1
- new package built with tito

%autochangelog
