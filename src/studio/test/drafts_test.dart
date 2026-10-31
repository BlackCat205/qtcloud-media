import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:qtcloud_media_studio/drafts_io.dart';

void main() {
  test('草稿读写与列表（注入目录）', () async {
    final dir = Directory(
      '${Directory.systemTemp.path}/qtm-studio-drafts-${DateTime.now().microsecondsSinceEpoch}',
    );
    await saveDraftIn(dir, 'a', '# A\n\n正文一\n');
    await saveDraftIn(dir, 'b', '# B\n\n正文二\n');
    expect(await listDraftsIn(dir), ['a', 'b']);
    expect(await readDraftIn(dir, 'a'), contains('正文一'));
    expect(await readDraftIn(dir, 'missing'), isNull);
    await dir.delete(recursive: true);
  });
}
