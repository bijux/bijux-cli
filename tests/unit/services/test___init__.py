# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the services init module."""

from __future__ import annotations

from unittest.mock import patch

import pytest

from bijux_cli.core.di import DIContainer
from bijux_cli.core.enums import OutputFormat
from bijux_cli.services import register_default_services


@pytest.mark.parametrize("output_format", [OutputFormat.JSON, OutputFormat.YAML])
@pytest.mark.parametrize("debug", [True, False])
@pytest.mark.parametrize("quiet", [True, False])
def test_register_default_services_100pct(
    output_format: OutputFormat, debug: bool, quiet: bool
) -> None:
    """Test that all default services are registered correctly with the DI container."""
    di = DIContainer()

    sentinel_audit_obj = object()
    with (
        patch(
            "bijux_cli.services.audit.get_audit_service",
            return_value=sentinel_audit_obj,
        ) as mock_get_audit,
        patch(
            "bijux_cli.infra.observability.Observability.__init__", return_value=None
        ) as mock_obs_init,
        patch(
            "bijux_cli.infra.observability.Observability.log",
            autospec=True,
            side_effect=lambda self, *a, **k: self,
        ) as mock_obs_log,
        patch(
            "bijux_cli.infra.telemetry.LoggingTelemetry.__init__", return_value=None
        ) as mock_tel_init,
        patch(
            "bijux_cli.infra.emitter.Emitter.__init__", return_value=None
        ) as mock_emitter_init,
        patch(
            "bijux_cli.infra.serializer.OrjsonSerializer.__init__", return_value=None
        ) as mock_orjson_init,
        patch(
            "bijux_cli.infra.serializer.PyYAMLSerializer.__init__", return_value=None
        ) as mock_yaml_init,
        patch(
            "bijux_cli.infra.process.ProcessPool.__init__", return_value=None
        ) as mock_process_init,
        patch(
            "bijux_cli.infra.retry.TimeoutRetryPolicy.__init__", return_value=None
        ) as mock_timeout_init,
        patch(
            "bijux_cli.infra.retry.ExponentialBackoffRetryPolicy.__init__",
            return_value=None,
        ) as mock_exp_init,
        patch(
            "bijux_cli.core.context.Context.__init__", return_value=None
        ) as mock_context_init,
        patch(
            "bijux_cli.services.config.Config.__init__", return_value=None
        ) as mock_config_init,
        patch(
            "bijux_cli.services.plugins.registry.Registry.__init__", return_value=None
        ) as mock_registry_init,
        patch(
            "bijux_cli.services.audit.DryRunAudit.__init__", return_value=None
        ) as mock_dry_audit_init,
        patch(
            "bijux_cli.services.audit.RealAudit.__init__", return_value=None
        ) as mock_real_audit_init,
        patch(
            "bijux_cli.services.docs.Docs.__init__", return_value=None
        ) as mock_docs_init,
        patch(
            "bijux_cli.services.doctor.Doctor.__init__", return_value=None
        ) as mock_doctor_init,
        patch(
            "bijux_cli.services.history.History.__init__", return_value=None
        ) as mock_history_init,
        patch(
            "bijux_cli.services.memory.Memory.__init__", return_value=None
        ) as mock_memory_init,
    ):
        from bijux_cli.contracts import (
            AuditProtocol,
            ConfigProtocol,
            ContextProtocol,
            DocsProtocol,
            DoctorProtocol,
            EmitterProtocol,
            HistoryProtocol,
            MemoryProtocol,
            ObservabilityProtocol,
            ProcessPoolProtocol,
            RegistryProtocol,
            RetryPolicyProtocol,
            SerializerProtocol,
            TelemetryProtocol,
        )
        import bijux_cli.core.context as core_context
        import bijux_cli.infra.emitter as infra_emitter
        import bijux_cli.infra.observability as infra_obs
        import bijux_cli.infra.process as infra_process
        import bijux_cli.infra.retry as infra_retry
        import bijux_cli.infra.serializer as infra_serializer
        import bijux_cli.infra.telemetry as infra_tel
        import bijux_cli.services.audit as svc_audit
        import bijux_cli.services.config as svc_config
        import bijux_cli.services.docs as svc_docs
        import bijux_cli.services.doctor as svc_doctor
        import bijux_cli.services.history as svc_history
        import bijux_cli.services.memory as svc_memory
        import bijux_cli.services.plugins.registry as svc_registry

        register_default_services(
            di, debug=debug, output_format=output_format, quiet=quiet
        )

        mock_obs_init.assert_called_once_with(debug=debug)
        obs_inst = di.resolve(infra_obs.Observability)
        assert di.resolve(ObservabilityProtocol) is obs_inst
        assert mock_obs_log.call_count >= 1

        tel_inst = di.resolve(TelemetryProtocol)
        mock_tel_init.assert_called_once()
        assert mock_tel_init.call_args.kwargs == {"observability": obs_inst}

        emitter_inst = di.resolve(EmitterProtocol)
        mock_emitter_init.assert_called_once()
        assert mock_emitter_init.call_args.kwargs == {
            "telemetry": tel_inst,
            "format": output_format,
            "debug": debug,
            "quiet": quiet,
        }

        serializer_inst = di.resolve(SerializerProtocol)
        if output_format is OutputFormat.JSON:
            assert isinstance(serializer_inst, infra_serializer.OrjsonSerializer)
            mock_orjson_init.assert_called_once()
            assert mock_orjson_init.call_args.kwargs == {"telemetry": tel_inst}
        else:
            assert isinstance(serializer_inst, infra_serializer.PyYAMLSerializer)
            mock_yaml_init.assert_called_once()
            assert mock_yaml_init.call_args.kwargs == {"telemetry": tel_inst}

        proc_inst = di.resolve(ProcessPoolProtocol)
        mock_process_init.assert_called_once()
        assert mock_process_init.call_args.kwargs == {
            "observability": obs_inst,
            "telemetry": tel_inst,
        }

        retry_inst = di.resolve(RetryPolicyProtocol)
        mock_timeout_init.assert_called_once()
        assert mock_timeout_init.call_args.kwargs == {"telemetry": tel_inst}
        exp_inst = di.resolve(infra_retry.ExponentialBackoffRetryPolicy)
        mock_exp_init.assert_called_once()
        assert mock_exp_init.call_args.kwargs == {"telemetry": tel_inst}
        assert isinstance(retry_inst, infra_retry.TimeoutRetryPolicy)
        assert isinstance(exp_inst, infra_retry.ExponentialBackoffRetryPolicy)

        ctx_inst = di.resolve(ContextProtocol)
        mock_context_init.assert_called_once_with(di)
        cfg_inst = di.resolve(ConfigProtocol)
        mock_config_init.assert_called_once_with(di)

        reg_inst = di.resolve(RegistryProtocol)
        mock_registry_init.assert_called_once()
        assert mock_registry_init.call_args.args == (tel_inst,)

        dry_audit_inst = di.resolve(svc_audit.DryRunAudit)
        mock_dry_audit_init.assert_called_once()
        assert mock_dry_audit_init.call_args.args == (obs_inst, tel_inst)
        real_audit_inst = di.resolve(svc_audit.RealAudit)
        mock_real_audit_init.assert_called_once()
        assert mock_real_audit_init.call_args.args == (obs_inst, tel_inst)

        audit_inst = di.resolve(AuditProtocol)
        mock_get_audit.assert_called_once()
        assert mock_get_audit.call_args.kwargs == {
            "observability": obs_inst,
            "telemetry": tel_inst,
            "dry_run": False,
        }
        assert audit_inst is sentinel_audit_obj

        docs_inst = di.resolve(DocsProtocol)
        mock_docs_init.assert_called_once()
        assert mock_docs_init.call_args.kwargs == {
            "observability": obs_inst,
            "telemetry": tel_inst,
        }

        doctor_inst = di.resolve(DoctorProtocol)
        mock_doctor_init.assert_called_once_with()

        history_inst = di.resolve(HistoryProtocol)
        mock_history_init.assert_called_once()
        assert mock_history_init.call_args.kwargs == {
            "telemetry": tel_inst,
            "observability": obs_inst,
        }

        memory_inst = di.resolve(MemoryProtocol)
        mock_memory_init.assert_called_once_with()

        assert isinstance(obs_inst, infra_obs.Observability)
        assert isinstance(tel_inst, infra_tel.LoggingTelemetry)
        assert isinstance(emitter_inst, infra_emitter.Emitter)
        assert isinstance(proc_inst, infra_process.ProcessPool)
        assert isinstance(ctx_inst, core_context.Context)
        assert isinstance(cfg_inst, svc_config.Config)
        assert isinstance(reg_inst, svc_registry.Registry)
        assert isinstance(dry_audit_inst, svc_audit.DryRunAudit)
        assert isinstance(real_audit_inst, svc_audit.RealAudit)
        assert isinstance(docs_inst, svc_docs.Docs)
        assert isinstance(doctor_inst, svc_doctor.Doctor)
        assert isinstance(history_inst, svc_history.History)
        assert isinstance(memory_inst, svc_memory.Memory)
