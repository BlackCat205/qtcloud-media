/// 远程模式：provider 接入（桌面端）。
///
/// local 模式（默认）：纯本地渲染 + 剪贴板复制到公众号编辑器；
/// remote 模式（配置了 provider 后）：一键把正文交给 provider 推公众号草稿箱，
/// 账号托管在 provider 侧。设置存 `~/.config/qtcloud-media/studio.json`。
library;

import 'dart:convert';
import 'dart:io';

import 'package:http/http.dart' as http;

import 'paths_io.dart';

class RemoteSettings {
  final String provider;
  final String account;
  const RemoteSettings({required this.provider, required this.account});

  Map<String, dynamic> toJson() => {'provider': provider, 'account': account};

  factory RemoteSettings.fromJson(Map<String, dynamic> j) => RemoteSettings(
    provider: (j['provider'] as String?) ?? '',
    account: (j['account'] as String?) ?? '',
  );
}

File settingsFileIn(Directory configDir) =>
    File('${configDir.path}/qtcloud-media/studio.json');

Future<RemoteSettings?> loadSettingsIn(Directory configDir) async {
  final file = settingsFileIn(configDir);
  if (!await file.exists()) return null;
  try {
    final j = jsonDecode(await file.readAsString()) as Map<String, dynamic>;
    final s = RemoteSettings.fromJson(j);
    return s.provider.isEmpty ? null : s;
  } catch (_) {
    return null;
  }
}

Future<RemoteSettings?> loadSettings() =>
    loadSettingsIn(Directory(xdgConfigDir().path));

Future<void> saveSettingsIn(Directory configDir, RemoteSettings s) async {
  final file = settingsFileIn(configDir);
  await file.parent.create(recursive: true);
  await file.writeAsString(jsonEncode(s.toJson()));
}

Future<void> saveSettings(RemoteSettings s) =>
    saveSettingsIn(Directory(xdgConfigDir().path), s);

Future<List<String>> listAccounts(String provider) async {
  final resp = await http
      .get(Uri.parse('$provider/accounts'))
      .timeout(const Duration(seconds: 5));
  if (resp.statusCode != 200) {
    throw Exception('provider 返回 ${resp.statusCode}');
  }
  return (jsonDecode(resp.body) as List).cast<String>();
}

/// 推送到 provider。返回「动作 media_id」一句话。
Future<String> publishRemote({
  required String provider,
  required String account,
  required String markdown,
  required String title,
  String digest = '',
  String? theme,
  String? draft,
}) async {
  final body = <String, dynamic>{
    'account': account,
    'markdown': markdown,
    'title': title,
    'digest': digest,
  };
  if (theme != null && theme.isNotEmpty) body['theme'] = theme;
  if (draft != null && draft.isNotEmpty) body['draft'] = draft;
  final resp = await http
      .post(
        Uri.parse('$provider/publish'),
        headers: {'content-type': 'application/json'},
        body: jsonEncode(body),
      )
      .timeout(const Duration(seconds: 60));
  final text = resp.body;
  if (resp.statusCode != 200) throw Exception('provider 返回错误：$text');
  final v = jsonDecode(text) as Map<String, dynamic>;
  return '${v['action']}，media_id=${v['media_id']}';
}
