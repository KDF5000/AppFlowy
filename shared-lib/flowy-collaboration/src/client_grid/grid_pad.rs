use crate::entities::revision::{md5, RepeatedRevision, Revision};
use crate::errors::{internal_error, CollaborateError, CollaborateResult};
use crate::util::{cal_diff, make_delta_from_revisions};
use flowy_grid_data_model::entities::{Field, FieldOrder, Grid, RawRow, RepeatedFieldOrder, RowOrder};
use lib_infra::uuid;
use lib_ot::core::{OperationTransformable, PlainTextAttributes, PlainTextDelta, PlainTextDeltaBuilder};
use std::sync::Arc;

pub type GridDelta = PlainTextDelta;
pub type GridDeltaBuilder = PlainTextDeltaBuilder;

pub struct GridPad {
    pub(crate) grid: Arc<Grid>,
    pub(crate) delta: GridDelta,
}

impl GridPad {
    pub fn from_delta(delta: GridDelta) -> CollaborateResult<Self> {
        let s = delta.to_str()?;
        let grid: Grid = serde_json::from_str(&s)
            .map_err(|e| CollaborateError::internal().context(format!("Deserialize delta to grid failed: {}", e)))?;

        Ok(Self {
            grid: Arc::new(grid),
            delta,
        })
    }

    pub fn from_revisions(_grid_id: &str, revisions: Vec<Revision>) -> CollaborateResult<Self> {
        let folder_delta: GridDelta = make_delta_from_revisions::<PlainTextAttributes>(revisions)?;
        Self::from_delta(folder_delta)
    }

    pub fn create_row(&mut self, row: &RawRow) -> CollaborateResult<Option<GridChange>> {
        self.modify_grid(|grid| {
            let row_order = RowOrder {
                grid_id: grid.id.clone(),
                row_id: row.id.clone(),
                visibility: true,
            };
            grid.row_orders.push(row_order);
            Ok(Some(()))
        })
    }

    pub fn create_field(&mut self, field: &Field) -> CollaborateResult<Option<GridChange>> {
        self.modify_grid(|grid| {
            let field_order = FieldOrder {
                field_id: field.id.clone(),
                visibility: true,
            };
            grid.field_orders.push(field_order);
            Ok(Some(()))
        })
    }

    pub fn delete_rows(&mut self, row_ids: &[String]) -> CollaborateResult<Option<GridChange>> {
        self.modify_grid(|grid| {
            grid.row_orders.retain(|row_order| !row_ids.contains(&row_order.row_id));
            Ok(Some(()))
        })
    }

    pub fn delete_field(&mut self, field_id: &str) -> CollaborateResult<Option<GridChange>> {
        self.modify_grid(|grid| {
            match grid
                .field_orders
                .iter()
                .position(|field_order| field_order.field_id == field_id)
            {
                None => Ok(None),
                Some(index) => {
                    grid.field_orders.remove(index);
                    Ok(Some(()))
                }
            }
        })
    }

    pub fn md5(&self) -> String {
        md5(&self.delta.to_bytes())
    }

    pub fn grid_data(&self) -> Grid {
        let grid_ref: &Grid = &self.grid;
        grid_ref.clone()
    }

    pub fn delta_str(&self) -> String {
        self.delta.to_delta_str()
    }

    pub fn field_orders(&self) -> &RepeatedFieldOrder {
        &self.grid.field_orders
    }

    pub fn modify_grid<F>(&mut self, f: F) -> CollaborateResult<Option<GridChange>>
    where
        F: FnOnce(&mut Grid) -> CollaborateResult<Option<()>>,
    {
        let cloned_grid = self.grid.clone();
        match f(Arc::make_mut(&mut self.grid))? {
            None => Ok(None),
            Some(_) => {
                let old = json_from_grid(&cloned_grid)?;
                let new = json_from_grid(&self.grid)?;
                match cal_diff::<PlainTextAttributes>(old, new) {
                    None => Ok(None),
                    Some(delta) => {
                        self.delta = self.delta.compose(&delta)?;
                        Ok(Some(GridChange { delta, md5: self.md5() }))
                    }
                }
            }
        }
    }
}

fn json_from_grid(grid: &Arc<Grid>) -> CollaborateResult<String> {
    let json = serde_json::to_string(grid)
        .map_err(|err| internal_error(format!("Serialize grid to json str failed. {:?}", err)))?;
    Ok(json)
}

pub struct GridChange {
    pub delta: GridDelta,
    /// md5: the md5 of the grid after applying the change.
    pub md5: String,
}

pub fn make_grid_delta(grid: &Grid) -> GridDelta {
    let json = serde_json::to_string(&grid).unwrap();
    PlainTextDeltaBuilder::new().insert(&json).build()
}

pub fn make_grid_revisions(user_id: &str, grid: &Grid) -> RepeatedRevision {
    let delta = make_grid_delta(grid);
    let bytes = delta.to_bytes();
    let revision = Revision::initial_revision(user_id, &grid.id, bytes);
    revision.into()
}

impl std::default::Default for GridPad {
    fn default() -> Self {
        let grid = Grid {
            id: uuid(),
            field_orders: Default::default(),
            row_orders: Default::default(),
        };
        let delta = make_grid_delta(&grid);
        GridPad {
            grid: Arc::new(grid),
            delta,
        }
    }
}
