import 'package:flutter/services.dart';

/// 富文本复制（桌面）：经 GTK 剪贴板同时写 text/html 与纯文本，
/// 粘贴进公众号编辑器时保留排版。实现在 linux/runner/my_application.cc。
Future<void> copyRich(String html, String text) async {
  await const MethodChannel('qtcloud_media/clipboard')
      .invokeMethod('setRich', {'html': html, 'text': text});
}
