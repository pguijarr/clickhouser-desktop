import { ForwardedRef, useEffect, useImperativeHandle, useState } from "react";
import { useForm } from "react-hook-form";
import { useConnectionContext } from "../../../../contexts/useConnectionContext";
import { connectionRepo } from "../../../../lib/backend-repos";
import { Connection, ConnectionBody } from "../../../../lib/clickhouse-clients";
import { testConnection } from "../../../../lib/connections-helpers";
import { AppToaster } from "../../../../lib/toaster/AppToaster";
import { ConnectionDialogRef } from "./ConnectionDialog";

type Params = {
  onClose: () => void;
  ref: ForwardedRef<ConnectionDialogRef>;
};

export const useConnectionDialog = ({ onClose, ref }: Params) => {
  const { activeConnectionId, setActiveConnectionDisplay } =
    useConnectionContext();
  const [isOpen, setIsOpen] = useState(false);

  const [connection, setConnection] = useState<Connection | undefined>(
    undefined
  );

  const { control, handleSubmit, getValues, reset, watch } =
    useForm<ConnectionBody>({
      values: {
        name: connection?.name || "",
        host: connection?.host || "",
        port: connection?.port || 8123,
        secure: connection?.secure || false,
        database: connection?.database || "",
        username: connection?.username || "default",
        password: connection?.password || "",
        color: connection?.color ?? "#000000",
      },
      defaultValues: {
        name: "",
        host: "",
        port: 8123,
        secure: false,
        database: "",
        username: "default",
        password: "",
        color: "#000000",
      },
    });

  const onClickTest = async () => {
    try {
      const connection = getValues();
      await test(connection);
      AppToaster.top.success("Successfully connected");
    } catch (error) {
      AppToaster.top.error((error as Error).message);
    }
  };

  const onSubmit = handleSubmit(async (data) => {
    try {
      await test(data);
    } catch (error) {
      AppToaster.top.error((error as Error).message);
      return;
    }

    try {
      await save(data);
      AppToaster.top.success("The connection has been saved successfully");
      close();
      reset();
    } catch (error) {
      AppToaster.top.error("The connection could not be saved");
    }
  });

  const [tested, setTested] = useState(false);

  useEffect(() => {
    const subscription = watch(() => setTested(false));
    return () => subscription.unsubscribe();
  }, [watch]);

  const close = () => {
    setConnection(undefined);
    setTested(false);
    reset();
    onClose();
    setIsOpen(false);
  };

  const open = (connection?: Connection) => {
    setIsOpen(true);
    setConnection(connection);
  };

  const test = async (connection: ConnectionBody) => {
    return testConnection(connection)
      .then(() => void setTested(true))
      .catch(() => {
        setTested(false);
        throw new Error("Could not connect to the server");
      });
  };

  useImperativeHandle(ref, () => ({ open }), []);

  const save = async (connectionToSave: ConnectionBody) => {
    if (connection) {
      await connectionRepo.update(
        connection.id,
        connectionToSave as Connection
      );
      if (connection.id === activeConnectionId?.id) {
        setActiveConnectionDisplay(connectionToSave);
      }
    } else {
      await connectionRepo.insert(connectionToSave as Connection);
    }
    close();
  };

  return {
    close,
    connection,
    control,
    isOpen,
    onClickTest,
    onSubmit,
    tested,
  };
};
