/// 平台无关入口；具体实现按平台条件导入。
/// 桌面见 drafts_io.dart，Web 见 drafts_web.dart。
library;

Future<List<String>> listDrafts() async => const [];
Future<String?> readDraft(String name) async => null;
Future<void> saveDraft(String name, String content) async {}
Future<void> deleteDraft(String name) async {}
