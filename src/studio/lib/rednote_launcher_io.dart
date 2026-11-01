/// 调本机 CLI 生成小红书图集（生成逻辑单源在 CLI，studio 只做入口）。
/// 返回输出目录路径。
library;

import 'dart:io';

Future<String> runRednote(String draftName) async {
  final bin = await _findBin();
  final result = await Process.run(bin, ['note', 'export', draftName]);
  if (result.exitCode != 0) {
    final err = (result.stderr as String).trim();
    throw Exception(err.isEmpty ? 'qtcloud-media rednote 退出码 ${result.exitCode}' : err);
  }
  final pages = (result.stdout as String)
      .split('\n')
      .map((l) => l.trim())
      .where((l) => l.endsWith('.png'))
      .length;
  final first = (result.stdout as String)
      .split('\n')
      .map((l) => l.trim())
      .firstWhere((l) => l.endsWith('.png'), orElse: () => '');
  final dir = first.isEmpty ? '' : File(first).parent.path;
  return '$dir\n共 $pages 页';
}

/// 找 CLI：先 PATH，再 ~/.cargo/bin。
Future<String> _findBin() async {
  final home = Platform.environment['HOME'] ?? '';
  for (final candidate in ['qtcloud-media', '$home/.cargo/bin/qtcloud-media']) {
    try {
      final r = await Process.run(candidate, ['--version']);
      if (r.exitCode == 0) return candidate;
    } catch (_) {
      // 试下一个候选
    }
  }
  throw Exception('未找到 qtcloud-media CLI（PATH 或 ~/.cargo/bin），先安装：cargo install qtcloud-media-cli');
}
