/// 远程模式（Web 端）：无文件系统且 provider 未开 CORS，暂不支持。
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
