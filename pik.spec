# This will be the name of the package created. Needs to match the name of the specfile
Name: pik
	
# Version as given by upstream
Version: 0.6.0
	
# Iteration of the the package of this particular version
# Increase when you change the package without changing the
# version. Always append %%{?dist}, this takes care of adding
# the e.g. .f32 to the package
Release: 1%{?dist}
	
# Multiple licenses can be combined with logical operators, e.g.
	
# GPLv3 and MIT and LGPL
	
# If different parts of the code use different licenses, you should
	
# add a comment here to clarify which license covers what
	
License: MIT
	
# A short description of less than 80 characters
Summary: Process Interactive Kill is a tool that helps to find and kill process
	
# Upstream URL
	
Url: https://github.com/jacek-kurlit/%{name}
	
# URL where to obtain the sources, or instructions in a comment if
# no URL is used, e.g.
#
# Sources can be obtained by
# git clone https://pagure.io/copr-tito-quickdoc
# cd copr-tito-quickdoc
# tito build --tgz
	
# Sources can be obtained by
# git clone https://pagure.io/copr-tito-quickdoc
# cd copr-tito-quickdoc
# tito build --tgz
Source0: %{name}-%{version}.tar.gz
	
# Which arch the package is to be built for. Mainly useful to mark
# arch-less packages as such
BuildArch: noarch
	
	
# List of packages required for building this package
	
BuildRequires: rust
	
# List of packages required by the package at runtime
#TODO: do I need to add glib.c?
# Requires: ...
	
	
# Full description of the package
# Can be multiline, anything until the next section (%%prep) becomes part of
# the description text. Wrap at 80 characters. 
	
%description
Process Interactive Kill is a command line tool that helps to find and kill process.
It works like pkill command but search is interactive.
	
#-- PREP, BUILD & INSTALL -----------------------------------------------------#
	
# The %%prep stage is used to set up the build directories, unpack & copy the
# sources, apply patches etc.. Basically anything that needs to be done before
# running ./configure in the usual ./configure, make, make install workflow
	
%prep
	
# often, the %%autosetup macro is all that is needed here. This will unpack &
# copy the sources to the correct directories and apply patches. If your source
# tarball does not extract to a directory of the same name, you can specify
# the directory using the -n <dir> switch. You can also pass the -p option of
# the patch utility
	
%autosetup
	
	
# The %%build stage is used to build the software. Most common build commands
# have macros that take care of setting the appropriate environment, directories,
# flags, etc., so for './configure', you'd use %%{configure}, for 'make' %%{make_build},
# for 'python setup.py build' %%{py3_build} etc.
# This stage contains everything that needs to be done in the source directory before
# installing the software on a target system
	
%build
	
%rust_build
	
	
# the %%install stage is used to install the software. This
# uses the actual installation paths using %%{buildroot} as the root, i.e.
# %%{buildroot}/usr/share becomes /usr/share when the package is installed on
# a real system.
# There are RPM macros for most standard paths (e.g. %%{_sysconfdir} for /etc,
# %%{_bindir} for /usr/bin and so on), try to use those instead of hardcoding
# the paths. This avoids errors and makes it easier to adapt to filesystem changes
	
%install
	
%rust_install
	
	
#-- FILES ---------------------------------------------------------------------#
	
# The files section list every file contained in the package, pretty much
# the list of files created in the %%install section. There are a number of
# special flags, like %%doc, %%license or %%dir that tell RPM what kind of
# file it is dealing with.
%files
	
%doc README.md
	
%license LICENSE
	
%{_bindir}/pik
	
	
#-- CHANGELOG -----------------------------------------------------------------#
# The changelog is the last section of the specfile. Everything after this is
# treated as part of the changelog.
# Entries should follow the format given below and be separated by one empty line.
# A * marks the beginning of a new entry and is followed by date, author and package
# version. Lines beginning with - after that list the changes contained in the package.
	
%changelog
