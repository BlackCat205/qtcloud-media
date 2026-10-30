/// 主题加载（Web 端）：无文件系统，用内置默认主题。
library;

import 'theme.dart';

Future<ThemeStyles> loadThemeStyles() async => const ThemeStyles();
