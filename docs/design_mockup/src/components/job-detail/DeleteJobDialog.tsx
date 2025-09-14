import { useState } from "react";
import { Button } from "../ui/button";
import { Checkbox } from "../ui/checkbox";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "../ui/dialog";

interface DeleteJobDialogProps {
  isOpen: boolean;
  jobName: string;
  onClose: () => void;
  onConfirm: (deleteFiles: boolean) => void;
}

export function DeleteJobDialog({ isOpen, jobName, onClose, onConfirm }: DeleteJobDialogProps) {
  const [deleteFiles, setDeleteFiles] = useState(true);

  const handleConfirm = () => {
    onConfirm(deleteFiles);
  };

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Delete Job: {jobName}?</DialogTitle>
          <DialogDescription>
            This action cannot be undone. This will permanently delete the job and optionally remove all associated files.
          </DialogDescription>
        </DialogHeader>
        
        <div className="py-4">
          <div className="flex items-center space-x-2">
            <Checkbox
              id="delete-files"
              checked={deleteFiles}
              onCheckedChange={(checked) => setDeleteFiles(checked === true)}
            />
            <label htmlFor="delete-files" className="text-sm">
              Also delete files (project and scratch folders)
            </label>
          </div>
        </div>
        
        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button variant="destructive" onClick={handleConfirm}>
            Delete
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}