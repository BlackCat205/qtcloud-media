/// 平台无关入口；具体实现按平台条件导入。
/// 桌面见 theme_loader_io.dart，Web 见 theme_loader_web.dart。
library;

import 'theme.dart';

Future<ThemeStyles> loadThemeStyles() async => const ThemeStyles();
