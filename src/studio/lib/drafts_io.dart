/// 草稿读写（桌面端）。与 CLI（Rust drafts.rs）共用同一目录：
/// `$XDG_STATE_HOME/qtcloud-media/drafts/<name>.md`。
/// meta（title/media_id 等）由 CLI 的 publish 维护，studio 只读不改；
/// 删除草稿时一并清掉 meta，不留孤儿文件。
library;

import 'dart:io';

import 'paths_io.dart';

Directory draftsDir() => Directory('${xdgStateDir().path}/drafts');

Future<List<String>> listDraftsIn(Directory dir) async {
  if (!await dir.exists()) return const [];
  final names = await dir
      .list()
      .where((e) => e is File && e.path.endsWith('.md'))
      .map((e) => e.uri.pathSegments.last.replaceAll('.md', ''))
      .toList();
  names.sort();
  return names;
}

Future<List<String>> listDrafts() => listDraftsIn(draftsDir());

Future<String?> readDraftIn(Directory dir, String name) async {
  final file = File('${dir.path}/$name.md');
  if (!await file.exists()) return null;
  return file.readAsString();
}

Future<String?> readDraft(String name) => readDraftIn(draftsDir(), name);

Future<void> saveDraftIn(Directory dir, String name, String content) async {
  await dir.create(recursive: true);
  await File('${dir.path}/$name.md').writeAsString(content);
}

Future<void> saveDraft(String name, String content) =>
    saveDraftIn(draftsDir(), name, content);

Future<void> deleteDraftIn(Directory dir, String name) async {
  final md = File('${dir.path}/$name.md');
  if (await md.exists()) await md.delete();
  final meta = File('${dir.path}/$name.meta.json');
  if (await meta.exists()) await meta.delete();
}

Future<void> deleteDraft(String name) => deleteDraftIn(draftsDir(), name);
