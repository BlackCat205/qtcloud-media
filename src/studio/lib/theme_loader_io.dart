/// 主题加载（桌面端）：按 CLI 同一查找顺序读 XDG 主题，
/// 找不到回退内置默认。
library;

import 'dart:io';

import 'paths_io.dart';
import 'theme.dart';

Future<ThemeStyles> loadThemeStyles() async {
  final data = xdgDataDir();
  final config = xdgConfigDir();
  for (final dir in [data, config]) {
    final file = File('${dir.path}/themes/default.toml');
    if (await file.exists()) {
      return ThemeStyles.fromToml(await file.readAsString());
    }
  }
  return const ThemeStyles();
}
