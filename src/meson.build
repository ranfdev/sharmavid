global_conf = configuration_data()
global_conf.set_quoted('APP_ID', application_id)
global_conf.set_quoted('PKGDATADIR', pkgdatadir)
global_conf.set_quoted('PROFILE', profile)
global_conf.set_quoted('VERSION', version + version_suffix)
global_conf.set_quoted('GETTEXT_PACKAGE', gettext_package)
global_conf.set_quoted('LOCALEDIR', localedir)
config = configure_file(
  input: 'config.rs.in',
  output: 'config.rs',
  configuration: global_conf
)
# Copy the config.rs output to the source directory.
run_command(
  'cp',
  meson.build_root() / 'src' / 'config.rs',
  meson.source_root() / 'src' / 'config.rs',
  check: true
)

rust_sources = files(
  'config.rs',
  'main.rs',
  'widgets/window.rs',
  'widgets/video_row.rs',
  'widgets/thumbnail.rs',
  'widgets/video_page.rs',
  'widgets/channel_page.rs',
  'widgets/remote_image.rs',
  'widgets/mod.rs',
  'glib_utils/any_list_model.rs',
  'glib_utils/any_gobject.rs',
  'glib_utils/mod.rs',
  'invidious/core.rs',
  'invidious/client.rs',
  'invidious/mod.rs'
)

sources = [rust_sources, cargo_sources]

custom_target(
  'cargo-build',
  build_by_default: true,
  input: sources,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: bindir,
  depends: resources,
  command: [
    cargo_script,
    meson.build_root(),
    meson.source_root(),
    '@OUTPUT@',
    profile,
    meson.project_name(),
  ]
)
