/// 平台无关入口；具体实现按平台条件导入。
/// 桌面见 remote_io.dart，Web 见 remote_web.dart。
library;

class RemoteSettings {
  final String provider;
  final String account;
  const RemoteSettings({required this.provider, required this.account});
}

Future<RemoteSettings?> loadSettings() async => null;
Future<void> saveSettings(RemoteSettings s) async {}
Future<List<String>> listAccounts(String provider) async => const [];
Future<String> publishRemote({
  required String provider,
  required String account,
  required String markdown,
  required String title,
  String digest = '',
  String? theme,
  String? draft,
}) async => throw UnsupportedError('Web 端暂不支持远程推送');
