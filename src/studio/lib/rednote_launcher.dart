/// 平台无关入口：调本机 CLI 生成小红书图集。桌面见 rednote_launcher_io.dart。
library;

Future<String> runRednote(String draftName) async =>
    throw UnsupportedError('Web 端不支持生成小红书图集，请用 CLI：qtcloud-media rednote');
