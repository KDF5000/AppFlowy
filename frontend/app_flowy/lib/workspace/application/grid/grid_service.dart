import 'package:flowy_sdk/dispatch/dispatch.dart';
import 'package:flowy_sdk/protobuf/flowy-error/errors.pb.dart';
import 'package:flowy_sdk/protobuf/flowy-folder-data-model/view.pb.dart';
import 'package:flowy_sdk/protobuf/flowy-grid-data-model/grid.pb.dart';
import 'package:dartz/dartz.dart';
import 'package:flowy_sdk/protobuf/flowy-grid-data-model/meta.pb.dart';

class GridService {
  Future<Either<Grid, FlowyError>> openGrid({required String gridId}) async {
    await FolderEventSetLatestView(ViewId(value: gridId)).send();

    final payload = GridId(value: gridId);
    return GridEventGetGridData(payload).send();
  }

  Future<Either<void, FlowyError>> createRow({required String gridId}) {
    return GridEventCreateRow(GridId(value: gridId)).send();
  }

  Future<Either<RepeatedRow, FlowyError>> getRows({required String gridId, required List<RowOrder> rowOrders}) {
    final payload = QueryRowPayload.create()
      ..gridId = gridId
      ..rowOrders = RepeatedRowOrder(items: rowOrders);
    return GridEventGetRows(payload).send();
  }

  Future<Either<RepeatedField, FlowyError>> getFields({required String gridId, required List<FieldOrder> fieldOrders}) {
    final payload = QueryFieldPayload.create()
      ..gridId = gridId
      ..fieldOrders = RepeatedFieldOrder(items: fieldOrders);
    return GridEventGetFields(payload).send();
  }
}