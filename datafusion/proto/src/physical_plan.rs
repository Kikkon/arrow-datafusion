// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use crate::protobuf::logical_plan_node::LogicalPlanType::CustomScan;
use crate::protobuf::CustomTableScanNode;
use crate::{
    from_proto::{self, parse_expr},
    protobuf::{
        self, listing_table_scan_node::FileFormatType,
        logical_plan_node::LogicalPlanType, LogicalExtensionNode, LogicalPlanNode,
    },
    to_proto,
};
use arrow::datatypes::{Schema, SchemaRef};
use datafusion::datasource::TableProvider;
use datafusion::execution::FunctionRegistry;
use datafusion::physical_plan::ExecutionPlan;
use datafusion::{
    datasource::{
        file_format::{
            avro::AvroFormat, csv::CsvFormat, parquet::ParquetFormat, FileFormat,
        },
        listing::{ListingOptions, ListingTable, ListingTableConfig, ListingTableUrl},
        view::ViewTable,
    },
    datasource::{provider_as_source, source_as_provider},
    prelude::SessionContext,
};
use datafusion_common::{context, Column, DataFusionError};
use datafusion_expr::{
    logical_plan::{
        Aggregate, CreateCatalog, CreateCatalogSchema, CreateExternalTable, CreateView,
        CrossJoin, Distinct, EmptyRelation, Extension, Join, JoinConstraint, JoinType,
        Limit, Projection, Repartition, Sort, SubqueryAlias, TableScan, Values, Window,
    },
    Expr, LogicalPlan, LogicalPlanBuilder,
};
use prost::bytes::BufMut;
use prost::Message;
use std::fmt::Debug;
use std::sync::Arc;

pub trait PhysicalExtensionCodec: Debug + Send + Sync {
    fn try_decode(
        &self,
        buf: &[u8],
        inputs: &[Arc<dyn ExecutionPlan>],
        registry: &dyn FunctionRegistry,
    ) -> Result<Arc<dyn ExecutionPlan>, DataFusionError>;

    fn try_encode(
        &self,
        node: Arc<dyn ExecutionPlan>,
        buf: &mut Vec<u8>,
    ) -> Result<(), DataFusionError>;
}
