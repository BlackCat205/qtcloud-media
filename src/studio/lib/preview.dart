/// 平台无关入口；具体实现按平台条件导入。
/// Web 见 preview_web.dart，桌面见 preview_desktop.dart。
library;
import 'package:flutter/material.dart';

import 'md.dart';
import 'theme.dart';

Widget buildPreview(List<Block> blocks, ThemeStyles theme) =>
    const SizedBox.expand(child: Center(child: Text('当前平台不支持预览')));
