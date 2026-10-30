/// 平台无关入口；具体实现按平台条件导入。
/// Web 见 copy_web.dart，桌面见 copy_desktop.dart。
library;

Future<void> copyRich(String html, String text) async {
  throw UnimplementedError('当前平台不支持富文本复制');
}
