import driverdesk_ai


def test_version_is_exposed() -> None:
    assert driverdesk_ai.version() == driverdesk_ai.__version__ == "0.0.0"
