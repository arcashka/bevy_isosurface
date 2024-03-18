use bevy::{
    prelude::*,
    render::{
        render_graph::{self, RenderGraphContext, RenderLabel},
        render_resource::{ComputePassDescriptor, PipelineCache},
        renderer::RenderContext,
    },
};

use super::{CalculateIsosurfaces, IsosurfaceBindGroupsCollection, IsosurfaceComputePipelines};

use crate::assets::IsosurfaceAssetsStorage;

#[derive(Default)]
pub struct IsosurfaceComputeNode;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct IsosurfaceComputeNodeLabel;

impl render_graph::Node for IsosurfaceComputeNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let compute_pipelines = world.resource::<IsosurfaceComputePipelines>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let (
            Some(find_vertices_pipeline),
            Some(connect_vertices_pipeline),
            Some(prepare_indirect_buffer_pipeline),
        ) = (
            pipeline_cache.get_compute_pipeline(compute_pipelines.find_vertices_pipeline),
            pipeline_cache.get_compute_pipeline(compute_pipelines.connect_vertices_pipeline),
            pipeline_cache.get_compute_pipeline(compute_pipelines.prepare_indirect_buffer_pipeline),
        )
        else {
            return Ok(());
        };
        let encoder = render_context.command_encoder();
        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());

        let assets = world.resource::<IsosurfaceAssetsStorage>();
        let isosurfaces = world.resource::<CalculateIsosurfaces>();
        let bind_groups = world.resource::<IsosurfaceBindGroupsCollection>();

        for isosurface in isosurfaces.iter() {
            if !isosurface.ready {
                continue;
            }
            let Some(asset) = assets.get(&isosurface.asset_id) else {
                error!("missing isosurface asset");
                return Ok(());
            };
            let Some(bind_group) = bind_groups.get(&isosurface.asset_id) else {
                error!("missing isosurface compute bind group");
                return Ok(());
            };
            let density = asset.grid_density;
            pass.set_bind_group(0, bind_group, &[]);
            pass.set_pipeline(find_vertices_pipeline);
            pass.dispatch_workgroups(density.x, density.y, density.z);
            pass.set_pipeline(connect_vertices_pipeline);
            pass.dispatch_workgroups(density.x, density.y, density.z);
            pass.set_pipeline(prepare_indirect_buffer_pipeline);
            pass.dispatch_workgroups(1, 1, 1);
            info!("dispatch done");
        }
        Ok(())
    }
}
