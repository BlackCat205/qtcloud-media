import 'dart:js_interop';

@JS('copyRichText')
external JSPromise _copyRichText(String html, String text);

/// 富文本复制（Web）：text/html + text/plain，见 web/index.html。
Future<void> copyRich(String html, String text) async {
  await _copyRichText(html, text).toDart;
}
