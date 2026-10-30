/// XDG 路径解析（桌面端）。与 CLI（Rust paths.rs）同一条规则：
/// 环境变量优先且须为绝对路径，否则回退家目录；应用目录统一叫 qtcloud-media。
library;

import 'dart:io';

Directory _base(String varName, List<String> fallbackParts) {
  final env = Platform.environment[varName];
  final value = (env != null && env.startsWith('/')) ? env : null;
  final base = value ??
      [Platform.environment['HOME'] ?? '', ...fallbackParts].join('/');
  return Directory('$base/qtcloud-media');
}

Directory xdgConfigDir() => _base('XDG_CONFIG_HOME', ['.config']);
Directory xdgDataDir() => _base('XDG_DATA_HOME', ['.local', 'share']);
Directory xdgCacheDir() => _base('XDG_CACHE_HOME', ['.cache']);
