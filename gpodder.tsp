import "@typespec/http";
import "@typespec/versioning";

using TypeSpec.Http;
using TypeSpec.Versioning;

@versioned(Versions)
@service({
  title: "GPodder Service",
})
namespace GPodderService {
  enum Versions {
    v2_00: "v2.0",
    v2_01: "v2.1",
    v2_02: "v2.2",
    v2_03: "v2.3",
    v2_04: "v2.4",
    v2_05: "v2.5",
    v2_06: "v2.6",
    v2_07: "v2.7",
    v2_08: "v2.8",
    v2_09: "v2.9",
    v2_10: "v2.10",
    v2_11: "v2.11",
  }

  @tag("Authentication")
  namespace Authentication {
    @added(Versions.v2_10)
    @route("/api/2/auth/{username}/login.json")
    @post op login(
      @path username: string,
    ):
      | { @statusCode status: 200, @header("Set-Cookie") cookie: string }
      | { @statusCode status: 400 }
      | { @statusCode status: 401 };

    @added(Versions.v2_10)
    @route("/api/2/auth/{username}/logout.json")
    @post op logout(
      @path username: string,
    ):
      | { @statusCode status: 200 }
      | { @statusCode status: 400 };
  }

  @tag("Directory")
  namespace Directory {}

  @tag("Suggestions")
  namespace Suggestions {}

  @tag("Device")
  namespace Devices {
    alias DeviceType = "desktop" | "laptop" | "mobile" | "server" | "other";

    @added(Versions.v2_00)
    model Device {
      id: string;
      caption: string;
      type: DeviceType;
      subscriptions: integer;
    }

    @added(Versions.v2_00)
    model DeviceUpdate {
      caption: string;
      type: DeviceType;
    }

    @added(Versions.v2_00)
    @useAuth(BasicAuth | ApiKeyAuth<ApiKeyLocation.header, "Cookie">)
    @route("/api/2/devices/{username}/{device_id}.json")
    @post op update(
      @path username: string,
      @path device_id: string,
      @body update: DeviceUpdate,
    ):
      | { @statusCode status: 200 }
      | { @statusCode status: 400 }
      | { @statusCode status: 401 };

    @added(Versions.v2_00)
    @useAuth(BasicAuth | ApiKeyAuth<ApiKeyLocation.header, "Cookie">)
    @route("/api/2/devices/{username}.json")
    @get op list(
      @path username: string,
    ):
      | { @statusCode status: 200, @body body: Device[] };

    @added(Versions.v2_03)
    @useAuth(BasicAuth | ApiKeyAuth<ApiKeyLocation.header, "Cookie">)
    @route("/api/2/devices/{username}/{device_id}.json")
    @get op updates(
      @path username: string,
      @path device_id: string,
    ):
      | { @statusCode status: 200 };
  }

  @tag("Suggestions")
  namespace Suggestions {}

  @tag("Episode")
  namespace Episode {
    @added(Versions.v2_00)
    model BaseAction<NAME> {
      action: NAME;
    }

    @added(Versions.v2_00)
    union Action {
      Add: BaseAction<"add">;
      Remove: BaseAction<"remove">;
      Download: BaseAction<"download">;
      Delete: BaseAction<"delete">;
      Play: BaseAction<"play"> & { started?: integer, position?: integer, total?: integer };
      New: BaseAction<"new">;
      Flattr: BaseAction<"flattr">;
    }

    @added(Versions.v2_00)
    model EpisodeActions {
      actions: Action[];
      timestamp: integer;
    }

    /**
     * Timestamps: The result is a list of all episode actions that were
     * uploaded since the timestamp given in the since parameter
     * (regardless of the action timestamp itself). The timestamp SHOULD
     * be the value returned by the previous episode retrieve request. If no
     * since value is given, ALL episode actions for the given user are
     * returned. Please note that this could be a potentially long list of
     * episode actions, so clients SHOULD provide a since value whenever
     * possible (e.g. when uploads have been taken place before).
     *
     * @param podcast The URL of a Podcast feed; if set, only actions for
     * episodes of the given podcast are returned
     * @param device A Device ID; if set, only actions for the given device
     * are returned
     * @param since Only episode actions since the given timestamp are returned
     * @param aggregated  If true, only the latest actions is returned for each
     * episode (added in 2.1)
     */
    @added(Versions.v2_00)
    @useAuth(BasicAuth | ApiKeyAuth<ApiKeyLocation.header, "Cookie">)
    @route("/api/2/episodes/{username}.json")
    @get op actions(
      @path username: string,
      @query podcast?: string,
      @query device?: string,
      @query since?: integer,
      @added(Versions.v2_01)
      @query aggregated?: boolean,
    ):
      | { @statusCode status: 200, @body body: EpisodeActions };
  }

  @tag("Subscriptions")
  namespace Subscriptions {}

  @tag("Podcast")
  namespace Podcast {}

  @tag("Settings")
  namespace Settings {}

  @tag("Favorites")
  namespace Favorites {}

  @tag("Synchronization")
  namespace Synchronization {}

  @tag("Parametrization")
  namespace Parametrization {}
}
