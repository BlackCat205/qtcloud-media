/// 草稿读写（Web 端）：无文件系统，暂不支持草稿接力。
library;

Future<List<String>> listDrafts() async => const [];
Future<String?> readDraft(String name) async => null;
Future<void> saveDraft(String name, String content) async {}
Future<void> deleteDraft(String name) async {}
